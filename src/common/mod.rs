// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use WordId;
use Word;

pub struct CommonPrefixIter<'a, T> {
    word_id: WordId,
    word_len: usize,
    word: Word<'a>,
    node: T,
    finished: bool,
}

impl<'a, T: NodeTraverse> CommonPrefixIter<'a, T> {
    pub fn new(word: Word<'a>, root: T) -> Self {
        let mut it = CommonPrefixIter {
            word_id: 0,
            word_len: word.len(),
            word: word,
            node: root,
            finished: false,
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
        self.finished = true;
    }

    fn next_child(&mut self) -> bool {
        self.node.jump(&mut self.word).map(|_| self.word_id += self.node.id_offset()).is_some()
    }
}

impl<'a, T: NodeTraverse> Iterator for CommonPrefixIter<'a, T> {
    type Item = (WordId, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            let prefix_len = self.word_len - self.word.len();
            let item = (self.word_id, prefix_len);
            self.word_id += 1;
            self.go_to_next_common_prefix();
            Some(item)
        }
    }
}

pub trait NodeTraverse {
    fn is_terminal(&self) -> bool;
    fn id_offset(&self) -> u32;
    fn jump(&mut self, word: &mut Word) -> Option<()>;
}
