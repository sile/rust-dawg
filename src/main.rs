extern crate dawg;

use std::env;
use std::process;
use std::io;
use std::io::BufRead;
use dawg::build::Builder;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} OUTPUT_FILE", args[0]);
        process::exit(1);
    }

    let stdin = io::stdin();
    let output_file = &args[1];
    if let Err(e) = Builder::build_file(stdin.lock().lines(), output_file) {
        println!("[ERROR] Can't build DAWG: reason={}", e);
        process::exit(1);
    }
    println!("DONE");
}
