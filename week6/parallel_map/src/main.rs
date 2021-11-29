use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    // TODO: implement parallel map!
    let (sender_to_func,receiver_from_vec) =crossbeam_channel::unbounded();
    let (sender_to_output,receiver_from_func)=crossbeam_channel::unbounded();

    let mut threads=Vec::new();
    for _ in 0..num_threads{
        let receiver=receiver_from_vec.clone();
        let sender=sender_to_output.clone();
        threads.push(thread::spawn(move ||{
            while let Ok((idx,num))=receiver.recv() {
                sender.send((idx,f(num))).expect("send to output fail");
            }
        }));
    }
    
    let len=input_vec.len();
    for i in 0..len{
        if let Some(input_num)=input_vec.pop(){
            sender_to_func.send((len-i-1,input_num)).expect("send to func fail");
        }
    }
    
    drop(sender_to_func);
    drop(sender_to_output);
    
    while let Ok((idx,next_num))=receiver_from_func.recv(){
        output_vec[idx]=next_num;
    }

    for thread in threads{
        thread.join().expect("panic in thread");
    }
    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
