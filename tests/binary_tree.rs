// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

extern crate dawg;

use dawg::binary_tree::Builder;

#[test]
fn build() {
    let trie =
        Builder::new()
        .build(words().into_iter().map(|w| Ok(w) ))
        .unwrap_or_else(|e| panic!(e.to_string()) );
    assert_eq!(words().len(), trie.len());
}

#[test]
fn search_common_prefix() {
    let trie = Builder::new().build(words().into_iter().map(|w| Ok(w) )).ok().unwrap();

    assert_eq!(0, trie.search_common_prefix("hoge").count());

    assert_eq!(vec![(0,"abc")],
               trie.search_common_prefix("abc").collect::<Vec<_>>());

    assert_eq!(vec![(4,"cd"), (5,"cddr")],
               trie.search_common_prefix("cddrr").collect::<Vec<_>>());
}

fn words() -> Vec<String> {
    vec![
        "abc",
        "b",
        "bbb",
        "car",
        "cd",
        "cddr",
        "cdr",
        ].iter().map(|w| w.to_string() ).collect()
}
