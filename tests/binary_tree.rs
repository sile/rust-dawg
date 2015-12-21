// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

extern crate dawg;

use dawg::binary_tree::Builder;

#[test]
fn build() {
    let mut b = Builder::new();
    for w in words().iter() {
        assert!(b.insert(w.bytes()).is_ok());
    }
    assert_eq!(words().len(), b.finish().len());
}

#[test]
fn search_common_prefix() {
    let trie = words()
                   .iter()
                   .fold(Builder::new(), |mut b, w| {
                       b.insert(w.bytes()).ok().unwrap();
                       b
                   })
                   .finish();

    assert_eq!(0, trie.search_common_prefix("hoge".bytes()).count());

    assert_eq!(vec![(0, 3)],
               trie.search_common_prefix("abc".bytes()).collect::<Vec<_>>());

    assert_eq!(vec![(4, 2), (5, 4)],
               trie.search_common_prefix("cddrr".bytes()).collect::<Vec<_>>());
}

fn words() -> [&'static str; 7] {
    ["abc", "b", "bbb", "car", "cd", "cddr", "cdr"]
}
