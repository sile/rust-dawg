use std::path::Path;
use std::fs::{self,File};
use std::io::Result as IoResult;
use std::io::Read;
use std::io::Write;
use std::io::BufWriter;
use std::io::{Seek, SeekFrom};
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::Display;
use bit_vec::BitVec;
use byteorder::{ByteOrder, NativeEndian};

type Memo = HashMap<Rc<trie::Node>, u32>; // TODO: eq比較で十分

pub struct Builder;

struct DoubleArray<W> {
    memo: Memo,
    allocator: Allocator,
    node_writer: W,
    ext_position: usize,
    ext_writer: W,
}

struct Allocator {
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

    pub fn allocate(&mut self, arcs: Vec<u8>) -> u32 {
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
        }  else {
            self.nexts[index - self.offset]
        }
    }

    fn can_allocate(&mut self, index: usize, arcs: &[u8]) -> bool {
        if self.is_allocated(index) {
            return false // already used
        }
        arcs.iter().all(|a| self.get(index + *a as usize) != -1 )
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

impl Builder {
    pub fn build_file<Input, P>(words: Input, output_path: P) -> IoResult<()>
        where Input: Iterator<Item = IoResult<String>>,
              P: AsRef<Path> + Display
    {
        let trie = try!(trie::Builder::new().build(words));
        let temp_node_path = format!("{}.temp.node", output_path);
        let temp_ext_path = format!("{}.temp.ext", output_path);
        {
            let temp_node_file = try!(File::create(&temp_node_path));
            let temp_ext_file = try!(File::create(&temp_ext_path));

            let mut da =  DoubleArray {
                memo: HashMap::new(),
                allocator: Allocator::new(),
                node_writer: BufWriter::new(temp_node_file),
                ext_writer: BufWriter::new(temp_ext_file),
                ext_position: 0,
            };

            let node = Node::new(0, &trie.root);
            try!(da.build(Rc::new(trie.root), node));
        }
        {
            let mut temp_node_file = try!(File::open(&temp_node_path));
            let mut temp_ext_file = try!(File::open(&temp_ext_path));
            let mut dawg_file = try!(File::create(output_path));

            // TODO: iterative
            let mut buf = vec![0; 8];
            NativeEndian::write_u32(&mut buf, try!(temp_node_file.metadata()).len() as u32);
            NativeEndian::write_u32(&mut buf[4..], try!(temp_node_file.metadata()).len() as u32);
            try!(temp_node_file.read_to_end(&mut buf));
            try!(temp_ext_file.read_to_end(&mut buf));
            try!(dawg_file.write_all(&buf));
        }
        try!(fs::remove_file(&temp_node_path));
        try!(fs::remove_file(&temp_ext_path));
        Ok(())
    }
}

struct Node {
    children: Vec<u8>,
    index: u32,
    base: u32,
    sibling_total: u32,
    chck: u8,
    node_type: u8,
    is_terminal: bool,
}

impl Node {
    pub fn new(parent_base_idx: u32, trie: &trie::Node) -> Self {
        Node {
            base: 0,
            index: parent_base_idx + trie.label as u32,
            sibling_total: trie.sibling_total as u32,
            is_terminal: trie.is_terminal,
            chck: trie.label,
            node_type: match trie.sibling_total {
                n if n < 0x100 => 0,
                n if n < 0x10000 => 1,
                n if n < 0x1000000 => 2,
                _ => 3,
            },
            children: Vec::new(),
        }
    }

    pub fn is_child_acceptable(&self) -> bool {
        let capacity =
            match self.node_type {
                0 => 2,
                1 => 1,
                _ => 0,
            };
        capacity > self.children.len()
    }

    pub fn add_child(&mut self, label: u8) {
        self.children.push(label);
    }
}

impl<W: Write+Seek> DoubleArray<W> {
    pub fn build(&mut self, mut trie: Rc<trie::Node>, mut node: Node) -> IoResult<()> {
        let mut children;
        let mut is_memoized;
        loop {
            children = trie.collect_children(); // TODO: reverse(?)
            children.reverse();
            is_memoized = trie.child.as_ref().map(|c| self.memo.contains_key(c) ).unwrap_or(false);
            if is_memoized {
                break
            }
            if children.len() != 1 {
                break
            }
            if children[0].is_terminal {
                break
            }
            if ! node.is_child_acceptable() {
                break
            }
            node.add_child(children[0].label);
            trie = children[0].clone();
        }

        if is_memoized {
            let &base = trie.child.as_ref().map(|c| self.memo.get(c) ).unwrap().unwrap();
            return self.write_node(node, Some(base))
        }

        if children.is_empty() {
            return self.write_node(node, None)
        }

        let base_idx = self.allocator.allocate(children.iter().map(|x| x.label ).collect()); // TODO: passed iterator
        self.memo.insert(trie.child.as_ref().unwrap().clone(), base_idx);
        try!(self.write_node(node, Some(base_idx)));
        for child in children.iter() {
            try!(self.build(child.clone(), Node::new(base_idx, &child)));
        }
        Ok(())
    }

    fn write_node(&mut self, mut node: Node, base: Option<u32>) -> IoResult<()> {
        node.base = base.unwrap_or(node.base);

        let n: u64 =
            mask(node.base        as u64,  0, 29) +
            mask(node.node_type   as u64, 29,  2) +
            mask(node.is_terminal as u64, 31,  1) +
            mask(node.chck        as u64, 32,  8);

        let n =
            match node.node_type {
                0 => {
                    n + mask(*node.children.get(0).unwrap_or(&0) as u64, 40, 8) +
                        mask(*node.children.get(1).unwrap_or(&0) as u64, 48, 8) +
                        mask(node.sibling_total                 as u64, 56, 8)
                },
                1 => {
                    n + mask(*node.children.get(0).unwrap_or(&0) as u64, 40,  8) +
                        mask(node.sibling_total                 as u64, 48, 16)
                },
                2 => {
                    n + mask(node.sibling_total                 as u64, 40, 24)
                },
                3 => {
                    let m = node.sibling_total;
                    let mut buf = [0; 4];
                    NativeEndian::write_u32(&mut buf, m);

                    try!(self.ext_writer.write_all(&buf));
                    self.ext_position += 4;
                    n + mask((self.ext_position - 4) as u64, 40, 24)
                },
                _ => unreachable!(),
            };
        {
            let mut buf = [0; 8];
            NativeEndian::write_u64(&mut buf, n);
            try!(self.node_writer.seek(SeekFrom::Start((node.index * 8) as u64)));
            try!(self.node_writer.write_all(&buf));
        }
        Ok(())
    }
}

fn mask(x: u64, offset: usize, size: usize) -> u64 {
    (x & ((1 << size) - 1)) << offset
}

mod trie {
    use std::io::Result as IoResult;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::cmp::PartialEq;
    use std::hash::Hash;
    use std::hash::Hasher;
    use std::mem;
    use std::ops::Deref;

    //type Memo = HashSet<Rc<Node>>;
    type Memo = HashMap<Rc<Node>, Rc<Node>>;

    pub struct Trie {
        pub root: Node,
    }

    #[derive(Eq)]
    pub struct Node {
        pub child: Option<Rc<Node>>,
        pub sibling: Option<Rc<Node>>,
        pub child_total: u32, // amount of child side nodes
        pub sibling_total: u32, // amount of sibling side nodes
        pub label: u8,
        pub is_terminal: bool,
    }

    impl PartialEq for Node {
        fn eq(&self, other: &Node) -> bool {
            (self.child.as_ref().map(addr) == other.child.as_ref().map(addr) &&
             self.sibling.as_ref().map(addr) == other.sibling.as_ref().map(addr) &&
             self.label == other.label &&
             self.is_terminal == other.is_terminal)
        }
    }

    impl Hash for Node {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.label.hash(state);
            self.is_terminal.hash(state);
            self.child.as_ref().map(addr).hash(state);
            self.sibling.as_ref().map(addr).hash(state);
        }
    }

    fn addr<T>(x: &Rc<T>) -> usize {
        unsafe {
            mem::transmute(x.deref())
        }
    }

    pub struct Builder {
        memo: Memo,
    }

    impl Builder {
        pub fn new() -> Self {
            Builder {memo: HashMap::new()}
        }

        pub fn build<Input>(&mut self, words: Input) -> IoResult<Trie> where Input: Iterator<Item = IoResult<String>> {
            let mut root = Node::new(0);

            for word in words {
                // TODO: sortness check
                let word = try!(word);
                self.insert(&mut root, word.as_bytes());
            }

            println!("nodes: {}, {}, {}", self.memo.len(), root.child.is_some(), root.sibling.is_some());
            let root = self.share(Rc::new(root));
            self.memo.clear();
            Ok(Trie{root: Rc::try_unwrap(root).ok().unwrap()})
        }

        fn insert(&mut self, parent: &mut Node, word: &[u8]) {
            match parent.child.take() {
                None => {
                    self.push_child(parent, word);
                },
                Some(mut child) => {
                    if word.is_empty() || word[0] != child.label {
                        parent.child = Some(self.share(child));
                        self.push_child(parent, word);
                    } else {
                        self.insert(Rc::get_mut(&mut child).unwrap(), &word[1..]);
                        parent.child = Some(child);
                    }
                },
            }
        }

        fn push_child(&mut self, parent: &mut Node, word: &[u8]) {
            if word.is_empty() {
                parent.is_terminal = true;
            } else {
                let mut child = Node::new(word[0]);
                self.push_child(&mut child, &word[1..]);
                child.sibling = parent.child.take();
                parent.child = Some(Rc::new(child));
            }
        }

        fn share(&mut self, node: Rc<Node>) -> Rc<Node> {
            if let Some(n) = self.memo.get(&node) {
                return n.clone()
            }

            let mut node = Rc::try_unwrap(node).ok().unwrap();
            node.sibling = node.sibling.map(|n| self.share(n) );
            node.child = node.child.map(|n| self.share(n) );

            node.child_total = node.calc_child_total();
            node.sibling_total = node.calc_sibling_total();
            let node = Rc::new(node);
            if let Some(n) = self.memo.get(&node) {
                return n.clone()
            }

            self.memo.insert(node.clone(), node.clone());
            node
        }
    }

    impl Node {
        pub fn new(label: u8) -> Self {
            Node {
                label: label,
                is_terminal: false,
                child: None,
                sibling: None,
                child_total: 0,
                sibling_total: 0,
            }
        }

        pub fn calc_child_total(&mut self) -> u32 {
            self.child.as_ref().map(|n| n.is_terminal as u32 + n.child_total + n.sibling_total ).unwrap_or(0)
        }

        pub fn calc_sibling_total(&mut self) -> u32 {
            self.sibling.as_ref().map(|n| n.is_terminal as u32 + n.child_total + n.sibling_total ).unwrap_or(0)
        }

        // TODO: returns iterator
        pub fn collect_children(&self) -> Vec<Rc<Node>> {
            let mut v = Vec::new();
            match self.child.as_ref() {
                None        => v,
                Some(mut c) => {
                    v.push(c.clone());
                    while let Some(x) = c.sibling.as_ref() {
                        v.push(x.clone());
                        c = x;
                    }
                    v
                }
            }
        }
    }
}
