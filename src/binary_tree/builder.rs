// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::collections::HashMap;
use std::rc::Rc;
use std::io::Result as IoResult;
use std::io::Error as IoError;
use std::io::ErrorKind;
use binary_tree::Node;
use binary_tree::Trie;

pub struct Builder {
    memo: Memo,
}

type Memo = HashMap<Rc<Node>, Rc<Node>>;

impl Builder {
    pub fn new() -> Self {
        Builder {
            memo: Memo::new(),
        }
    }

    pub fn build<Words>(mut self, words: Words) -> IoResult<Trie> where Words: Iterator<Item = IoResult<String>> {
        let mut root = Node::new(0);
        let mut prev = None;

        for word in words {
            let word = try!(word);
            try!(validate_input_order(prev.as_ref(), &word));
            self.insert(&mut root, word.as_bytes());
            prev = Some(word);
        }
        self.share_children(&mut root);
        root.fix();

        Ok(Trie::new(root))
    }

    fn insert(&mut self, parent: &mut Node, word: &[u8]) {
        if word.is_empty() || parent.child.as_ref().map_or(true, |c| c.label != word[0] ) {
            self.add_new_child(parent, word);
        } else {
            let child = parent.child.as_mut().unwrap();
            self.insert(Rc::get_mut(child).unwrap(), &word[1..]);
        }
    }

    fn add_new_child(&mut self, parent: &mut Node, word: &[u8]) {
        if word.is_empty() {
            parent.is_terminal = true;
        } else {
            let mut child = Node::new(word[0]);
            self.add_new_child(&mut child, &word[1..]);
            child.sibling = parent.child.take().map(|c| self.share(c) );
            parent.child = Some(Rc::new(child));
        }
    }

    fn share(&mut self, mut node: Rc<Node>) -> Rc<Node> {
        if let Some(n) = self.memo.get(&node) {
            return n.clone()
        }
        self.share_children(Rc::get_mut(&mut node).unwrap());
        if let Some(n) = self.memo.get(&node) {
            return n.clone()
        }
        Rc::get_mut(&mut node).unwrap().fix();
        self.memo.insert(node.clone(), node.clone());
        node
    }

    fn share_children(&mut self, node: &mut Node) {
        node.sibling = node.sibling.take().map(|n| self.share(n) );
        node.child = node.child.take().map(|n| self.share(n) );
    }
}

fn validate_input_order(prev: Option<&String>, curr: &String) -> IoResult<()> {
    if prev.map_or(true, |p| p < curr) {
        Ok(())
    } else {
        let msg = format!("The input is not sorted: previous_word={:?}, current_word={:?}", prev.unwrap(), curr);
        Err(IoError::new(ErrorKind::InvalidInput, msg))
    }
}
