// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

extern crate dawg;

use std::env;
use std::process;
use std::io;
use std::io::BufRead;
use dawg::double_array::Trie;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} INDEX_FILE", args[0]);
        process::exit(1);
    }

    let stdin = io::stdin();
    let index_file = &args[1];
    let trie = Trie::load(index_file)
        .unwrap_or_else(|e| {
                       println!("[ERROR] Can't load DAWG index: path={}, reason={}", index_file, e);
                       process::exit(1);
                   });
    println!(">>>");
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_else(|e| {
            println!("[ERROR] Can't read stdin: reason={}", e);
            process::exit(1);
        });
        for (word_id, prefix) in trie.search_common_prefix(&line) {
            println!("  [{}] {}", word_id, prefix);
        }
        println!("");
    }
}
