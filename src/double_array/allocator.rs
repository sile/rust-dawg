// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use bit_vec::BitVec;

pub struct Allocator {
    head: usize,
    bits: BitVec,
    nexts: Vec<isize>,
    prevs: Vec<isize>,
    offset: usize,
}

const BUFFER_SIZE: usize = 89120;

impl Allocator {
    pub fn new() -> Self {
        let mut prevs = vec![0; BUFFER_SIZE];
        let mut nexts = vec![0; BUFFER_SIZE];
        for i in 0..BUFFER_SIZE {
            nexts[i] = i as isize + 1;
            prevs[i] = i as isize - 1;
        }
        Allocator {
            head: 0x100,
            bits: BitVec::from_elem(BUFFER_SIZE, false),
            nexts: nexts,
            prevs: prevs,
            offset: 0,
        }
    }

    pub fn allocate(&mut self, arcs: &[u8]) -> u32 {
        assert!(arcs.len() > 0);

        let front = arcs[0];
        let head = self.head;
        let mut curr = self.get_next(head);
        loop {
            let base = curr - front as isize;
            if base > 0 && self.can_allocate(base as usize, &arcs[1..]) {
                self.allocate_impl(base as usize, &arcs);
                return base as u32;
            }
            curr = self.get_next(curr as usize);
        }
    }

    fn allocate_impl(&mut self, index: usize, arcs: &[u8]) {
        if self.offset <= index {
            self.bits.set(index - self.offset, true);
        }

        let base = index;
        for arc in arcs.iter() {
            let index = base + *arc as usize;
            if self.offset <= index {
                self.get(index);

                let j = index - self.offset;
                let prev = self.prevs[j];
                let next = self.nexts[j];
                assert!(prev != -1);
                assert!(next != -1);

                self.prevs[j] = -1; // TOOD: => None
                self.nexts[j] = -1;

                if self.head == index {
                    self.head = next as usize;
                }
                if self.offset <= prev as usize {
                    self.nexts[prev as usize - self.offset] = next;
                }
                if self.offset <= next as usize {
                    self.get(next as usize);
                    self.prevs[next as usize - self.offset] = prev;
                }
            }
        }
    }

    fn get_next(&mut self, index: usize) -> isize {
        self.get(index)
    }

    fn get(&mut self, index: usize) -> isize {
        if self.offset + BUFFER_SIZE <= index {
            self.shift();
            self.get(index)
        } else {
            self.nexts[index - self.offset]
        }
    }

    fn can_allocate(&mut self, index: usize, arcs: &[u8]) -> bool {
        if self.is_allocated(index) {
            return false; // already used
        }
        arcs.iter().all(|a| self.get(index + *a as usize) != -1)
    }

    fn is_allocated(&mut self, index: usize) -> bool {
        if self.offset > index {
            true
        } else {
            if self.offset + BUFFER_SIZE <= index {
                self.shift();
                self.is_allocated(index)
            } else {
                self.bits.get(index - self.offset).unwrap()
            }
        }
    }

    fn shift(&mut self) {
        let mut new_offset = self.head;
        while new_offset < self.offset + BUFFER_SIZE - 0x100 * 2 {
            new_offset = self.nexts[new_offset - self.offset] as usize;
        }

        let delta = new_offset - self.offset;
        let use_len = BUFFER_SIZE - delta;

        for i in 0..use_len {
            let x = self.bits[delta + i];
            self.bits.set(i, x);
        }
        for i in use_len..BUFFER_SIZE {
            self.bits.set(i, false);
        }

        self.offset = new_offset;
        for i in 0..use_len {
            let x = self.nexts[delta + i];
            let y = self.prevs[delta + i];
            self.nexts[i] = x;
            self.prevs[i] = y;
        }
        for i in use_len..BUFFER_SIZE {
            self.nexts[i] = (self.offset + i + 1) as isize;
            self.prevs[i] = (self.offset + i - 1) as isize;
        }

        self.head = self.offset;
        while self.head < self.offset + 0x100 {
            let next = self.nexts[self.head - self.offset];
            self.head = next as usize;
        }
    }
}
