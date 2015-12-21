// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::str;
use WordId;
use Char;

pub struct CommonPrefixIter<'a, T> {
    word_id: WordId,
    word: &'a [Char],
    offset: usize,
    node: T,
}

impl<'a, T: NodeTraverse> CommonPrefixIter<'a, T> {
    pub fn new(word: &'a str, root: T) -> Self {
        let mut it = CommonPrefixIter {
            word_id: 0,
            word: word.as_bytes(),
            offset: 0,
            node: root,
        };
        if !it.node.is_terminal() {
            it.go_to_next_common_prefix();
        }
        it
    }

    fn go_to_next_common_prefix(&mut self) {
        while self.next_child() {
            if self.node.is_terminal() {
                return;
            }
        }
        self.offset = self.word.len() + 1; // Set EOS
    }

    fn next_child(&mut self) -> bool {
        if self.offset == self.word.len() {
            return false;
        }

        self.node
            .jump_words(&self.word[self.offset..])
            .map(|read| {
                self.offset += read;
                self.word_id += self.node.id_offset();
            })
            .is_some()
    }
}

impl<'a, T: NodeTraverse> Iterator for CommonPrefixIter<'a, T> {
    type Item = (WordId, &'a str);

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

pub trait NodeTraverse {
    fn is_terminal(&self) -> bool;
    fn id_offset(&self) -> u32;
    fn jump_char(&mut self, ch: Char) -> bool;
    fn jump_words(&mut self, word: &[Char]) -> Option<usize> {
        if self.jump_char(word[0]) {
            Some(1)
        } else {
            None
        }
    }
}
