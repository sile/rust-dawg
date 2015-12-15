// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::str;
use std::path::Path;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::Write;
use std::io::BufWriter;
use std::io::Read;
use std::io::BufReader;
use byteorder::ByteOrder;
use byteorder::NativeEndian;
use WordId;

pub struct Trie {
    nodes: Vec<u64>,
    exts: Vec<u32>,
}

impl Trie {
    pub fn new(nodes: Vec<u64>, exts: Vec<u32>) -> Self {
        Trie {
            nodes: nodes,
            exts: exts,
        }
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut node = self.nodes[0];
        loop {
            count += is_terminal(node) as usize + self.id_offset(node) as usize;

            for i in 0x00.. {
                if i == 0xFF {
                    return count;
                }

                let label = (0xFF - i) as u8;
                if let Some(next) = self.next(node, label) {
                    node = next;
                    break;
                }
            }
        }
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
            node: &self.nodes[0],
            word: word.as_bytes(),
            nodes: &self.nodes,
            exts: &self.exts,
        };
        if !is_terminal(*it.node) {
            it.go_to_next_common_prefix();
        }
        it
    }

    pub fn load<P: AsRef<Path>>(index_file_path: P) -> IoResult<Self> {
        let mut r = BufReader::new(try!(File::open(index_file_path)));
        let node_count = try!(read_u32(&mut r)) / 8;
        let ext_count = try!(read_u32(&mut r)) / 4;

        let mut nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            nodes.push(try!(read_u64(&mut r)));
        }

        let mut exts = Vec::with_capacity(ext_count as usize);
        for _ in 0..ext_count {
            exts.push(try!(read_u32(&mut r)));
        }

        Ok(Trie::new(nodes, exts))
    }

    // TODO: add padding
    pub fn save<P: AsRef<Path>>(&self, index_file_path: P) -> IoResult<()> {
        let mut w = BufWriter::new(try!(File::create(index_file_path)));
        try!(write_u32(&mut w, self.nodes.len() as u32 * 8));
        try!(write_u32(&mut w, self.exts.len() as u32 * 4));
        for n in self.nodes.iter() {
            try!(write_u64(&mut w, *n));
        }
        for e in self.exts.iter() {
            try!(write_u32(&mut w, *e));
        }
        Ok(())
    }

    fn id_offset(&self, n: u64) -> u32 {
        let node_type = mask(n, 29, 2);
        match node_type {
            0 => mask(n, 56, 8) as u32,
            1 => mask(n, 48, 16) as u32,
            2 => mask(n, 40, 24) as u32,
            3 => self.exts[mask(n, 40, 24) as usize],
            _ => unreachable!(),
        }
    }

    fn next(&self, n: u64, label: u8) -> Option<u64> {
        let base = base(n) as usize;
        if self.nodes.len() <= base + label as usize {
            return None;
        }

        let next = self.nodes[(base + label as usize)];
        let chck = mask(next, 32, 8) as u8;
        if label == chck {
            Some(next)
        } else {
            None
        }
    }
}

fn read_u32<R: Read>(r: &mut R) -> IoResult<u32> {
    let mut buf = [0; 4];
    let size = try!(r.read(&mut buf));
    assert_eq!(size, buf.len());
    Ok(NativeEndian::read_u32(&buf))
}

fn read_u64<R: Read>(r: &mut R) -> IoResult<u64> {
    let mut buf = [0; 8];
    let size = try!(r.read(&mut buf));
    assert_eq!(size, buf.len());
    Ok(NativeEndian::read_u64(&buf))
}

fn write_u32<W: Write>(w: &mut W, n: u32) -> IoResult<()> {
    let mut buf = [0; 4];
    NativeEndian::write_u32(&mut buf, n);
    w.write_all(&mut buf)
}

fn write_u64<W: Write>(w: &mut W, n: u64) -> IoResult<()> {
    let mut buf = [0; 8];
    NativeEndian::write_u64(&mut buf, n);
    w.write_all(&mut buf)
}

fn base(n: u64) -> u32 {
    mask(n, 0, 29) as u32
}

fn is_terminal(n: u64) -> bool {
    mask(n, 31, 1) == 1
}

fn mask(n: u64, offset: usize, size: usize) -> u64 {
    (n >> offset) & ((1 << size) - 1)
}

pub struct CommonPrefixIter<'a, 'b> {
    word_id: WordId,
    offset: usize,
    node: &'a u64,
    word: &'b [u8],
    nodes: &'a Vec<u64>,
    exts: &'a Vec<u32>,
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
            if is_terminal(*self.node) {
                return;
            }
        }
        self.offset = self.word.len() + 1;
    }

    fn next_child(&mut self) -> bool {
        if self.offset == self.word.len() {
            return false;
        }

        if !self.check_encoded_children() {
            return false;
        }

        let label = self.word[self.offset];
        self.offset += 1;
        if let Some(next) = self.next_node(*self.node, label) {
            self.word_id += self.id_offset(*next);
            self.node = next;
            true
        } else {
            false
        }
    }

    fn check_encoded_children(&mut self) -> bool {
        let node_type = mask(*self.node, 29, 2);
        match node_type {
            0 => {
                let c0 = mask(*self.node, 40, 8) as u8;
                let c1 = mask(*self.node, 48, 8) as u8;
                if (c0 == 0 || self.word[self.offset] == c0) &&
                   (c1 == 0 || self.word.get(self.offset + 1).map_or(false, |x| *x == c1)) {
                    if c0 != 0 {
                        self.offset += 1;
                    }
                    if c1 != 0 {
                        self.offset += 1;
                    }
                    self.offset < self.word.len()
                } else {
                    false
                }
            }
            1 => {
                let c = mask(*self.node, 40, 8) as u8;
                if c == 0 || self.word[self.offset] == c {
                    if c != 0 {
                        self.offset += 1;
                    }
                    self.offset < self.word.len()
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    fn id_offset(&self, n: u64) -> u32 {
        let node_type = mask(n, 29, 2);
        match node_type {
            0 => mask(n, 56, 8) as u32,
            1 => mask(n, 48, 16) as u32,
            2 => mask(n, 40, 24) as u32,
            3 => self.exts[mask(n, 40, 24) as usize],
            _ => unreachable!(),
        }
    }

    fn next_node(&self, n: u64, label: u8) -> Option<&'a u64> {
        let base = base(n) as usize;
        if self.nodes.len() <= base + label as usize {
            return None;
        }

        let next = &self.nodes[(base + label as usize)];
        let chck = mask(*next, 32, 8) as u8;
        if label == chck {
            Some(next)
        } else {
            None
        }
    }
}
