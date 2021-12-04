mod request;
mod response;

use clap::Parser;
use rand::{Rng, SeedableRng};
use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;
use std::sync::{Arc, Mutex,RwLock};
use std::io::{ErrorKind};
use std::thread;
use std::time;
use std::collections::HashMap;
use std::net::{IpAddr};

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[clap(about = "Fun with load balancing")]
struct CmdOptions {
    #[clap(
        short,
        long,
        about = "IP/port to bind to",
        default_value = "0.0.0.0:1100"
    )]
    bind: String,
    #[clap(short, long, about = "Upstream host to forward requests to")]
    upstream: Vec<String>,
    #[clap(
        long,
        about = "Perform active health checks on this interval (in seconds)",
        default_value = "10"
    )]
    active_health_check_interval: usize,
    #[clap(
    long,
    about = "Path to send request to for active health checks",
    default_value = "/"
    )]
    active_health_check_path: String,
    #[clap(
        long,
        about = "Maximum number of requests to accept per IP per minute (0 = unlimited)",
        default_value = "0"
    )]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.

struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    #[allow(dead_code)]
    active_health_check_interval: usize,
    /// Where we should send requests when doing active health checks (Milestone 4)
    #[allow(dead_code)]
    active_health_check_path: String,
    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    #[allow(dead_code)]
    max_requests_per_minute: usize,
    /// Addresses of servers that we are proxying to
    upstream_addresses: Vec<String>,
    offline_upstream: RwLock<(usize,Vec<bool>)>,
    rate_limit_counter: Mutex<HashMap<IpAddr, usize>>,
}

fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // println!("###############!");

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    println!("@@@@@@@@@@ {:?}",options);
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }
    // println!("###############!");

    // Start listening for connections
    let listener = match TcpListener::bind(&options.bind) {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);
    // println!("############### Listening for requests on {}", options.bind);

    // Handle incoming connections
    let upstream_len=options.upstream.len();
    let state = ProxyState {
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        offline_upstream:RwLock::new((upstream_len,vec![false;upstream_len])),
        rate_limit_counter: Mutex::new(HashMap::new()),
    };
    let shared_state=Arc::new(state);

    let state_for_active_check=shared_state.clone();
    thread::spawn(move || {
        active_health_check(&state_for_active_check);
    });
    let state_for_rate_check=shared_state.clone();
    thread::spawn(move ||{
        rate_limit_counter_refresher(&state_for_rate_check, 60);
    });

    let n_workers=10;
    let pool=ThreadPool::new(n_workers);
    for stream in listener.incoming() {
        if let Ok(mut _stream) = stream {
            // Handle the connection!
            if shared_state.max_requests_per_minute>0{
                let mut rate_limit_counter = shared_state.rate_limit_counter.lock().unwrap();
                let ip_addr=_stream.peer_addr().unwrap().ip();
                let counter=rate_limit_counter.entry(ip_addr).or_insert(0);
                *counter+=1;
                if *counter > shared_state.max_requests_per_minute{
                    request::read_from_stream(&mut _stream).ok();
                    let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
                    response::write_to_stream(&response, &mut _stream).unwrap();
                    send_response(&mut _stream, &response);
                    continue;
                }
            }
            let tmp_state=shared_state.clone();
            pool.execute(  move || {
                handle_connection(_stream, &tmp_state);
            });
        }
    }
}
fn rate_limit_counter_refresher(state: &ProxyState, interval: u64) {
    thread::sleep(time::Duration::from_secs(interval));
    let mut rate_limit_counter = state.rate_limit_counter.lock().unwrap();
    rate_limit_counter.clear();
}

fn check_server(state: &ProxyState , idx: usize, path: &String) -> Option<usize> {
    let upstream_ip = &state.upstream_addresses[idx];
    let mut stream = TcpStream::connect(upstream_ip).ok()?;
    let request = http::Request::builder()
            .method(http::Method::GET)
            .uri(path)
            .header("Host", upstream_ip)
            .body(Vec::new())
            .unwrap();

    let _ = request::write_to_stream(&request, &mut stream).ok()?;
    let res = response::read_from_stream(&mut stream, &http::Method::GET).ok()?;

    if res.status().as_u16() == 200 {
        return Some(200);
    } else {
        return None;
    }
}

fn active_health_check(state: &ProxyState) {
    let interval = state.active_health_check_interval as u64;
    let path = &state.active_health_check_path;
    loop {
        thread::sleep(time::Duration::from_secs(interval));
        let mut off = state.offline_upstream.write().unwrap();
        for idx in 0..off.1.len() {
            if check_server(&state, idx, path).is_some()  {
                // down -> up
                if off.1[idx] {
                    off.0 += 1;
                    off.1[idx] = false;
                }
            } else {
                // up -> down
                if !off.1[idx] {
                    off.0 -= 1;
                    off.1[idx] = true;
                }
            }
        }
    }
}

fn connect_to_upstream(state: &ProxyState) -> Result<TcpStream, std::io::Error> {
    let mut off=state.offline_upstream.write().unwrap();
    if off.0==0{
        return Err(std::io::Error::new(ErrorKind::Other,"After down, All the upstream servers are down!"))
    }

    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut upstream_idx = rng.gen_range(0, state.upstream_addresses.len());
    while off.1[upstream_idx]==true {
        let mut rng = rand::rngs::StdRng::from_entropy();
        upstream_idx = rng.gen_range(0, state.upstream_addresses.len());
    }
    let upstream_ip = &state.upstream_addresses[upstream_idx];

    TcpStream::connect(upstream_ip).or_else(|err| {
        log::error!("Failed to connect to upstream {}: {}", upstream_ip, err);
        println!("{}",off.0);

        if off.1[upstream_idx]==false{
            off.0-=1;
            off.1[upstream_idx]=true;
        }
        
        if off.0==0{
            return Err(std::io::Error::new(ErrorKind::Other,"After down, All the upstream servers are down!"))
        }else{
            for i in 0..off.1.len(){
                if off.1[i]==false {
                    return TcpStream::connect(&state.upstream_addresses[i])
                }
            }
        }
        Err(err)
    })

    // TODO: implement failover (milestone 3)
}

fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("{} <- {}", client_ip, response::format_response_line(&response));
    if let Err(error) = response::write_to_stream(&response, client_conn) {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

fn handle_connection(mut client_conn: TcpStream, state: &ProxyState) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state) {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response);
            return;
        }
    };
    let upstream_ip = client_conn.peer_addr().unwrap().ip().to_string();   

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn) {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response);
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn) {
            log::error!("Failed to send request to upstream {}: {}", upstream_ip, error);
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response);
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()) {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response);
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response);
        log::debug!("Forwarded response to client");
    }
}
