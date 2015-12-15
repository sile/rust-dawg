// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

use std::path::Path;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::Write;
use std::io::BufWriter;
use std::io::Read;
use std::io::BufReader;
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
