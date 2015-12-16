// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use bit_vec::BitVec;

pub struct Allocator {
    head: usize,
    base_used: BitVec,
    node_used: BitVec,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            head: 0,
            base_used: BitVec::new(),
            node_used: BitVec::new(),
        }
    }

    // maybe unnecessary
    pub fn fix(self, nexts: &mut Vec<u64>) {
        for i in 0..nexts.len() {
            if !self.node_used.get(i).unwrap() {
                nexts[i] = 0;
            }
        }
    }

    pub fn allocate(&mut self, arcs: &[u8], nexts: &mut Vec<u64>) -> u32 {
        assert!(arcs.len() > 0);

        let front = arcs[0];
        while get_next(self.head, nexts) < front as u32 {
            self.head = get_next(self.head, nexts) as usize;
        }
        let mut prev = self.head;
        let mut curr = get_next(prev, nexts);

        loop {
            let base = curr - front as u32;
            if self.can_allocate(base as usize, &arcs[1..]) {
                self.allocate_impl(base as usize, &arcs, prev, nexts);
                return base;
            }
            prev = curr as usize;
            curr = get_next(curr as usize, nexts);
        }
    }

    fn allocate_impl(&mut self, base: usize, arcs: &[u8], mut prev: usize, nexts: &mut Vec<u64>) {
        self.extend_if_needed(base + 0x100, nexts);
        self.base_used.set(base, true);
        for arc in arcs.iter() {
            let index = base + *arc as usize;
            self.node_used.set(index, true);

            while nexts[prev] as usize != index {
                prev = nexts[prev] as usize;
                assert!(prev < index);
            }

            let next = nexts[index] as usize;
            if self.head == index {
                self.head = next;
            }
            nexts[prev] = next as u64;
        }
    }

    fn can_allocate(&mut self, base: usize, arcs: &[u8]) -> bool {
        if self.base_used.get(base).unwrap_or(false) {
            return false; // already used
        }
        arcs.iter().all(|a| !self.node_used.get(base + *a as usize).unwrap_or(false))
    }

    fn extend_if_needed(&mut self, index: usize, nexts: &mut Vec<u64>) {
        if index < nexts.len() {
            return;
        }
        let size = nexts.len();
        for i in size..index + 1 {
            nexts.push(i as u64 + 1);
        }
        self.base_used.grow(index + 1 - size, false);
        self.node_used.grow(index + 1 - size, false);
    }
}

fn get_next(index: usize, nexts: &mut Vec<u64>) -> u32 {
    nexts.get(index).map(|n| *n as u32).unwrap_or(index as u32 + 1)
}
