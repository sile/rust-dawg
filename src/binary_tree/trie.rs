// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::str;
use WordId;
use binary_tree::Node;

pub struct Trie {
    root: Node,
}

impl Trie {
    pub fn new(root: Node) -> Self {
        Trie { root: root }
    }

    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn to_node(self) -> Node {
        self.root
    }

    pub fn contains(&self, word: &str) -> bool {
        self.get_id(word).is_some()
    }

    pub fn get_id(&self, word: &str) -> Option<WordId> {
        self.search_common_prefix(word).find(|m| word.len() == m.1.len()).map(|m| m.0)
    }

    pub fn search_common_prefix<'a, 'b>(&'a self, word: &'b str) -> CommonPrefixIter<'a, 'b> {
        let mut it = CommonPrefixIter {
            word_id: 0,
            offset: 0,
            node: &self.root,
            word: word.as_bytes(),
        };
        if !it.node.is_terminal {
            it.go_to_next_common_prefix();
        }
        it
    }
}

pub struct CommonPrefixIter<'a, 'b> {
    word_id: WordId,
    offset: usize,
    node: &'a Node,
    word: &'b [u8],
}

impl<'a, 'b> Iterator for CommonPrefixIter<'a, 'b> {
    type Item = (WordId, &'b str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.word.len() {
            None
        } else {
            let prefix = unsafe { str::from_utf8_unchecked(&self.word[0..self.offset]) };
            let item = (self.word_id, prefix);
            self.word_id += 1;
            self.go_to_next_common_prefix();
            Some(item)
        }
    }
}

impl<'a, 'b> CommonPrefixIter<'a, 'b> {
    fn go_to_next_common_prefix(&mut self) {
        while self.next_child() {
            if self.node.is_terminal {
                return;
            }
        }
        self.offset = self.word.len() + 1;
    }

    fn next_child(&mut self) -> bool {
        if self.offset == self.word.len() {
            return false;
        }

        let label = self.word[self.offset];
        self.offset += 1;
        self.node
            .ref_children()
            .find(|c| c.label == label)
            .map(|c| {
                self.word_id += c.id_offset();
                self.node = c;
            })
            .is_some()
    }
}
