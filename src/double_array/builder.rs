// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::rc::Rc;
use std::collections::HashMap;
use binary_tree::NodeAddr;
use binary_tree::Trie as BinTreeTrie;
use binary_tree::Node as BinTreeNode;
use double_array::Base;
use double_array::Chck;
use double_array::Trie;
use double_array::allocator::Allocator;

pub struct Builder {
    memo: Memo,
    allocator: Allocator,
    nodes: Vec<u64>,
    exts: Vec<u32>,
}

type Memo = HashMap<NodeAddr, Base>;
type U24 = u32;

struct Node {
    chck: Chck,
    is_terminal: bool,
    index: u32,
    info: NodeInfo,
}

enum NodeInfo {
    Type0 {
        id_offset: u8,
        child1: Option<u8>,
        child2: Option<u8>,
    },
    Type1 {
        id_offset: u16,
        child: Option<u8>,
    },
    Type2 {
        id_offset: U24,
    },
    Type3 {
        id_offset: u32,
    },
}

impl Node {
    pub fn new(parent_base: Base, bt_node: &BinTreeNode) -> Self {
        Node {
            chck: bt_node.label,
            is_terminal: bt_node.is_terminal,
            index: parent_base + bt_node.label as u32,
            info: match bt_node.id_offset() {
                n if n < 0x100 => {
                    NodeInfo::Type0 {
                        id_offset: n as u8,
                        child1: None,
                        child2: None,
                    }
                }
                n if n < 0x10000 => {
                    NodeInfo::Type1 {
                        id_offset: n as u16,
                        child: None,
                    }
                }
                n if n < 0x1000000 => NodeInfo::Type2 { id_offset: n as U24 },
                n => NodeInfo::Type3 { id_offset: n },
            },
        }
    }

    pub fn try_add_child(&mut self, label: u8) -> bool {
        match &mut self.info {
            &mut NodeInfo::Type0{ref mut child1, ..} if child1.is_none() => {
                *child1 = Some(label);
                true
            }
            &mut NodeInfo::Type0{ref mut child2, ..} if child2.is_none() => {
                *child2 = Some(label);
                true
            }
            &mut NodeInfo::Type1{ref mut child, ..} if child.is_none() => {
                *child = Some(label);
                true
            }
            _ => false,
        }
    }
}

impl NodeInfo {
    pub fn type_id(&self) -> u8 {
        match self {
            &NodeInfo::Type0{..} => 0,
            &NodeInfo::Type1{..} => 1,
            &NodeInfo::Type2{..} => 2,
            &NodeInfo::Type3{..} => 3,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            memo: Memo::new(),
            allocator: Allocator::new(),
            nodes: Vec::new(),
            exts: Vec::new(),
        }
    }

    pub fn build(mut self, trie: BinTreeTrie) -> Trie {
        let bt_root = trie.to_node();
        let da_root = Node::new(0, &bt_root);
        self.build_impl(Rc::new(bt_root), da_root);
        self.allocator.fix(&mut self.nodes);
        Trie::new(self.nodes, self.exts)
    }

    fn build_impl(&mut self, mut bt_node: Rc<BinTreeNode>, mut da_node: Node) {
        let mut children: Vec<_>;
        let mut memo_key;
        let mut do_memoize;
        loop {
            if bt_node.child.is_none() {
                // empty children
                self.fix_node(da_node, 0);
                return;
            }

            memo_key = bt_node.child.as_ref().unwrap().addr();
            if let Some(base) = self.memo.get(&memo_key).cloned() {
                // have been memoized
                self.fix_node(da_node, base);
                return;
            }

            children = match Rc::try_unwrap(bt_node) {
                Ok(mut bt_node) => {
                    do_memoize = false; // not shared
                    bt_node.take_children().collect()
                }
                Err(bt_node) => {
                    do_memoize = true; // shared
                    bt_node.children().collect()
                }
            };
            children.reverse();

            if children.len() != 1 {
                break;
            }
            if children[0].is_terminal {
                break;
            }
            if !da_node.try_add_child(children[0].label) {
                break;
            }
            bt_node = children[0].clone();
        }

        let base = {
            let labels = children.iter().map(|c| c.label).collect::<Vec<_>>();
            self.allocator.allocate(&labels, &mut self.nodes)
        };
        if do_memoize {
            self.memo.insert(memo_key, base);
        }
        self.fix_node(da_node, base);
        for bt_child in children.into_iter() {
            let da_child = Node::new(base, &bt_child);
            self.build_impl(bt_child, da_child);
        }
    }

    fn fix_node(&mut self, node: Node, base: Base) {
        let n = mask(base as u64, 0, 29) + mask(node.info.type_id() as u64, 29, 2) +
                mask(node.is_terminal as u64, 31, 1) +
                mask(node.chck as u64, 32, 8);
        let n = match &node.info {
            &NodeInfo::Type0{id_offset, child1, child2} => {
                n + mask(child1.unwrap_or(0) as u64, 40, 8) +
                mask(child2.unwrap_or(0) as u64, 48, 8) +
                mask(id_offset as u64, 56, 8)
            }
            &NodeInfo::Type1{id_offset, child} => {
                n + mask(child.unwrap_or(0) as u64, 40, 8) + mask(id_offset as u64, 48, 16)
            }
            &NodeInfo::Type2{id_offset} => n + mask(id_offset as u64, 40, 24),
            &NodeInfo::Type3{id_offset} => {
                self.exts.push(id_offset);
                n + mask((self.exts.len() - 1) as u64, 40, 24)
            }
        };
        assert!((node.index as usize) < self.nodes.len());
        self.nodes[node.index as usize] = n;
    }
}

fn mask(x: u64, offset: usize, size: usize) -> u64 {
    (x & ((1 << size) - 1)) << offset
}
