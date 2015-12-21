// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::mem;
use std::collections::HashMap;
use std::rc::Rc;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error as FmtError;
use binary_tree::Node;
use binary_tree::Trie;
use EOS;
use Char;
use Word;

pub struct Builder {
    memo: Memo,
    root: Node,
}

type Memo = HashMap<Rc<Node>, Rc<Node>>;

impl Builder {
    pub fn new() -> Self {
        Builder {
            memo: Memo::new(),
            root: Node::new(EOS),
        }
    }

    pub fn insert(&mut self, word: Word) -> InsertResult {
        let mut root = mem::replace(&mut self.root, Node::new(EOS));
        try!(self.insert_word(&mut root, word));
        self.root = root;
        Ok(())
    }

    pub fn finish(mut self) -> Trie {
        let mut root = mem::replace(&mut self.root, Node::new(EOS));
        self.share_children(&mut root);
        root.fix();
        Trie::new(root)
    }

    fn insert_word(&mut self, parent: &mut Node, mut word: Word) -> InsertResult {
        match word.next() {
            Some(ch) if parent.child.as_ref().map_or(false, |c| c.ch == ch) => {
                let child = parent.child.as_mut().unwrap();
                self.insert_word(Rc::get_mut(child).unwrap(), word)
            }
            next_ch => self.add_new_child(parent, next_ch, word),
        }
    }

    fn add_new_child(&mut self,
                     parent: &mut Node,
                     ch: Option<Char>,
                     mut word: Word)
                     -> InsertResult {
        match ch {
            None => {
                parent.is_terminal = true;
                Ok(())
            }
            Some(ch) => {
                if ch == EOS {
                    return Err(InsertError::Eos);
                }
                if parent.child.as_ref().map_or(false, |c| c.ch > ch) {
                    return Err(InsertError::Unsorted);
                }
                let mut child = Node::new(ch);
                try!(self.add_new_child(&mut child, word.next(), word));
                child.sibling = parent.child.take().map(|c| self.share(c));
                parent.child = Some(Rc::new(child));
                Ok(())
            }
        }
    }

    fn share(&mut self, mut node: Rc<Node>) -> Rc<Node> {
        if let Some(n) = self.memo.get(&node) {
            return n.clone();
        }
        self.share_children(Rc::get_mut(&mut node).unwrap());
        if let Some(n) = self.memo.get(&node) {
            return n.clone();
        }
        Rc::get_mut(&mut node).unwrap().fix();
        self.memo.insert(node.clone(), node.clone());
        node
    }

    fn share_children(&mut self, node: &mut Node) {
        node.sibling = node.sibling.take().map(|n| self.share(n));
        node.child = node.child.take().map(|n| self.share(n));
    }
}

pub type InsertResult = Result<(), InsertError>;

#[derive(Debug)]
pub enum InsertError {
    Eos,
    Unsorted,
}

impl Error for InsertError {
    fn description(&self) -> &str {
        match self {
            &InsertError::Eos => "unexpected eos",
            &InsertError::Unsorted => "unsorted words",
        }
    }
}

impl Display for InsertError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let reason = match self {
            &InsertError::Eos => "a word can not have the EOS character",
            &InsertError::Unsorted => "words are not sorted",
        };
        f.write_str(reason)
    }
}
