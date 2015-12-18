// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::rc::Rc;
use WordId;
use binary_tree::Node;
use double_array::Trie as DoubleArrayTrie;
use double_array::Builder as DoubleArrayBuilder;
use common::CommonPrefixIter;
use common::NodeTraverse;

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

    pub fn to_double_array(self) -> DoubleArrayTrie {
        DoubleArrayBuilder::new().build(self)
    }

    pub fn contains(&self, word: &str) -> bool {
        self.get_id(word).is_some()
    }

    pub fn get_id(&self, word: &str) -> Option<WordId> {
        self.search_common_prefix(word).find(|m| word.len() == m.1.len()).map(|m| m.0)
    }

    pub fn search_common_prefix<'a>(&self, word: &'a str) -> CommonPrefixIter<'a, NodeTraverser> {
        CommonPrefixIter::new(word, NodeTraverser { node: Rc::new(self.root.clone()) })
    }
}

pub struct NodeTraverser {
    node: Rc<Node>,
}

impl NodeTraverse for NodeTraverser {
    fn is_terminal(&self) -> bool {
        self.node.is_terminal
    }

    fn id_offset(&self) -> u32 {
        self.node.id_offset()
    }

    fn jump_label(&mut self, label: u8) -> bool {
        self.node
            .children()
            .find(|c| c.label == label)
            .map(|c| self.node = c)
            .is_some()
    }
}
