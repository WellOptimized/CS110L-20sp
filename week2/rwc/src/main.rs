use std::env;
use std::process;

use std::io::prelude::*;
use std::io::{self, BufReader}; // For read_file_lines()
use std::fs::File; // For read_file_lines()
use std::collections::HashMap;

fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    // Be sure to delete the #[allow(unused)] line above
    let f = File::open(filename)?;
    let f = BufReader::new(f);
    let mut _v:Vec<String>=Vec::new();
    for line in f.lines() {
        match line{
            Ok(i)=>_v.push(i),
            Err(e)=>return Err(e),
        };
    }
    Ok(_v)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    let _vec:Vec<String> = match read_file_lines(filename){
        Ok(_i) => _i,
        Err(_e)=>return (),
    };

    let mut map:HashMap<String,i32>=HashMap::new();
    for i in &_vec{
        let split=i.split(" ");
        let vec:Vec<&str>=split.collect();
        // println!("{:?}",vec);
        for j in vec{
            let stat= map.entry((*j).to_string()).or_insert(0);
            *stat+=1;
        }
    }
    println!("{:?}",map);
}
