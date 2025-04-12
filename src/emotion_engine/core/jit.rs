use std::{collections::BTreeMap, ops::Range};

use bitvec::vec::BitVec;

pub struct Jit {
    jitted_instructions: BitVec<usize>,
    jitted_starts_map: BTreeMap<u32, u16>,
    jitted_starts: Box<[CacheIndex]>,
    cache: Vec<Code>,
}

#[derive(Clone, Copy)]
struct CacheIndex(u16);

const CODE_CACHE_MAX_SIZE: u16 = u16::MAX - 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CacheIndexView {
    NotCached,
    Cached(u16),
    NotJittable,
}

enum Entry {
    Vacant,
    Occupied(Box<Code>),
}

impl CacheIndex {
    pub fn not_cached() -> Self {
        CacheIndex(0)
    }

    pub fn not_jittable() -> Self {
        CacheIndex(u16::MAX)
    }

    pub fn cached(index: u16) -> Self {
        CacheIndex(index + 1)
    }

    pub fn view(self) -> CacheIndexView {
        match self.0 {
            0 => CacheIndexView::NotCached,
            u16::MAX => CacheIndexView::NotJittable,
            _ => CacheIndexView::Cached(self.0 - 1),
        }
    }
}

struct Code {
    address_range: Range<u32>,
}

const VIRTUAL_MEMORY_SIZE: usize = 0x1_0000_0000;
const INSTRUCTION_SIZE: usize = std::mem::size_of::<u32>();

impl Jit {
    pub fn new() -> Self {
        Jit {
            jitted_instructions: BitVec::from_vec(vec![
                0;
                VIRTUAL_MEMORY_SIZE
                    / INSTRUCTION_SIZE
                    / (std::mem::size_of::<usize>() * 8)
            ]),
            jitted_starts_map: BTreeMap::new(),
            jitted_starts: Box::new(
                [CacheIndex::not_cached(); VIRTUAL_MEMORY_SIZE / INSTRUCTION_SIZE],
            ),
            cache: Vec::new(),
        }
    }

    fn remove(&mut self, cache_index: u16) {
        let code = self.cache.swap_remove(cache_index as usize);
        if let Some(moved_code) = self.cache.last() {
            let moved_index = (self.cache.len() - 1) as u16;
            self.jitted_starts_map
                .insert(moved_code.address_range.start, moved_index)
                .unwrap();
            self.jitted_starts[moved_code.address_range.start as usize / INSTRUCTION_SIZE] =
                CacheIndex::cached(moved_index);
        }
        self.jitted_starts[code.address_range.start as usize / INSTRUCTION_SIZE] =
            CacheIndex::not_cached();
        self.jitted_starts_map
            .remove(&code.address_range.start)
            .unwrap();
        let start = self
            .jitted_starts_map
            .range(..code.address_range.start)
            .rev()
            .map(|(_, index)| {
                let earlier_code = &self.cache[*index as usize];
                earlier_code.address_range.end
            })
            .take_while(|earlier_code_end| *earlier_code_end > code.address_range.start)
            .max()
            .unwrap_or(code.address_range.start);
        let end = self
            .jitted_starts_map
            .range(code.address_range.start..code.address_range.end)
            .next()
            .map(|(start, _)| *start)
            .unwrap_or(code.address_range.end);
        if start < end {
            self.jitted_instructions
                [start as usize / INSTRUCTION_SIZE..end as usize / INSTRUCTION_SIZE]
                .fill(false);
        }
    }

    fn add(&mut self, code: Code) {
        if self.cache.len() as u16 == CODE_CACHE_MAX_SIZE {
            self.remove(0);
        }
        let cache_index = self.cache.len() as u16;
        self.jitted_starts[code.address_range.start as usize / INSTRUCTION_SIZE] =
            CacheIndex::cached(cache_index);
        self.jitted_starts_map
            .insert(code.address_range.start, cache_index);
        self.jitted_instructions[code.address_range.start as usize / INSTRUCTION_SIZE
            ..code.address_range.end as usize / INSTRUCTION_SIZE]
            .fill(true);
        self.cache.push(code);
    }

    #[inline(always)]
    pub fn invalidate_range(&mut self, range: Range<u32>) {
        if self.jitted_instructions[range.start as usize / INSTRUCTION_SIZE
            ..(range.end as usize).div_ceil(INSTRUCTION_SIZE)]
            .not_any()
        {
            return;
        }

        self.invalidate_range_slow(range);
    }

    fn invalidate_range_slow(&mut self, range: Range<u32>) {
        let to_remove = self
            .jitted_starts_map
            .range(..range.end)
            .rev()
            .take_while(|(_, index)| {
                let code = &self.cache[**index as usize];
                code.address_range.end > range.start
            })
            .map(|(_, index)| *index)
            .collect::<Vec<_>>();

        for index in to_remove {
            self.remove(index);
        }
    }
}
