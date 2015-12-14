// Copyright (c) 2015 Takeru Ohta <phjgt308@gmail.com>
//
// This software is released under the MIT License,
// see the LICENSE file at the top-level directory.

mod builder;
mod allocator;
mod trie;

pub type Base = u32;
pub type Chck = u8;

pub type Builder = builder::Builder;
pub type Trie = trie::Trie;
