// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use bit_vec::BitVec;

pub struct Allocator {
    head: usize,
    bits: BitVec,
    nexts: Vec<Option<u32>>,
    prevs: Vec<Option<u32>>,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            head: 0x100,
            bits: BitVec::from_elem(1, false),
            nexts: vec![Some(1)],
            prevs: vec![None],
        }
    }

    pub fn allocate(&mut self, arcs: &[u8]) -> u32 {
        assert!(arcs.len() > 0);

        // let mut count = 0;
        let front = arcs[0];
        let head = self.head;
        let mut curr = self.get_next(head);
        loop {
            let base = curr - front as u32;
            if self.can_allocate(base as usize, &arcs[1..]) {
                self.allocate_impl(base as usize, &arcs);
                // TODO: optimize
                // if count > 10 {
                //     let head = self.head;
                //     let next_head = self.get_next(head) as usize;
                //     self.head = next_head;
                // }
                return base;
            }
            // count += 1;
            curr = self.get_next(curr as usize);
        }
    }

    fn allocate_impl(&mut self, index: usize, arcs: &[u8]) {
        self.bits.set(index, true);

        let base = index;
        for arc in arcs.iter() {
            let index = base + *arc as usize;
            self.extend_if_needed(index);

            let prev = self.prevs[index].unwrap() as usize;
            let next = self.nexts[index].unwrap() as usize;

            self.prevs[index] = None;
            self.nexts[index] = None;

            if self.head == index {
                self.head = next;
            }
            self.nexts[prev] = Some(next as u32);

            self.extend_if_needed(next);
            self.prevs[next] = Some(prev as u32);
        }
    }

    fn get_next(&mut self, index: usize) -> u32 {
        self.extend_if_needed(index);
        self.nexts[index].unwrap()
    }

    fn can_allocate(&mut self, index: usize, arcs: &[u8]) -> bool {
        if self.is_allocated(index) {
            return false; // already used
        }
        arcs.iter().all(|a| self.nexts.get(index + *a as usize).map_or(true, |x| x.is_some()))
    }

    fn is_allocated(&mut self, index: usize) -> bool {
        self.extend_if_needed(index);
        self.bits.get(index).unwrap()
    }

    fn extend_if_needed(&mut self, index: usize) {
        if index < self.prevs.len() {
            return;
        }
        let size = self.prevs.len();
        for i in size..index + 1 {
            self.nexts.push(Some(i as u32 + 1));
            self.prevs.push(Some(i as u32 - 1));
        }
        self.bits.grow(index + 1 - size, false);
    }
}
