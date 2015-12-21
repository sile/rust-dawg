// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

extern crate bit_vec;
extern crate byteorder;

use std::str::Bytes;

pub mod binary_tree;
pub mod double_array;
pub mod common;

pub type Char = u8;
pub type WordId = u32;
pub type Word<'a> = Bytes<'a>;

pub const EOS: Char = 0 as Char;
