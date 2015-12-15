// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::path::Path;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::Write;
use std::io::BufWriter;
use byteorder::ByteOrder;
use byteorder::NativeEndian;

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
    pub fn load(&self) {
        unimplemented!()
    }

    pub fn save<P: AsRef<Path>>(&self, index_file_path: P) -> IoResult<()> {
        let mut w = BufWriter::new(try!(File::open(index_file_path)));

        let mut buf = [0; 4];
        NativeEndian::write_u32(&mut buf, self.nodes.len() as u32);
        try!(w.write_all(&mut buf));
        NativeEndian::write_u32(&mut buf, self.exts.len() as u32);
        try!(w.write_all(&mut buf));

        let mut buf = [0; 8];
        for n in self.nodes.iter() {
            NativeEndian::write_u64(&mut buf, *n);
            try!(w.write_all(&mut buf));
        }

        let mut buf = [0; 4];
        for e in self.exts.iter() {
            NativeEndian::write_u32(&mut buf, *e);
            try!(w.write_all(&mut buf));
        }
        Ok(())
    }
}
