use std::ops::Range;

use crate::emotion_engine::{
    bus::{BOOT_MEMORY_SIZE, MAIN_MEMORY_SIZE, SCRATCHPAD_SIZE},
    dmac::{MemoryOrScratchpadAddress, MemoryOrScratchpadAddressView},
};

const ADDRESSES: usize = (MAIN_MEMORY_SIZE + BOOT_MEMORY_SIZE + SCRATCHPAD_SIZE) / 4;
const FIRST_MAIN_MEMORY_INDEX: u32 = 0;
const FIRST_BOOT_MEMORY_INDEX: u32 = FIRST_MAIN_MEMORY_INDEX + MAIN_MEMORY_SIZE as u32 / 4;
const FIRST_SCRATCHPAD_INDEX: u32 = FIRST_BOOT_MEMORY_INDEX + BOOT_MEMORY_SIZE as u32 / 4;
const MAX_CACHED: u16 = u16::MAX - 1;

#[derive(Clone, Copy, PartialEq, Eq)]
struct AddressIndex(u32);

struct Jit {
    address_cache_index: Box<[CacheIndex]>,
    cache: Vec<Code>,
}

#[derive(Clone, Copy)]
struct CacheIndex(u16);

enum CacheIndexView {
    NotCached,
    Cached(u16),
}

impl From<CacheIndexView> for CacheIndex {
    fn from(value: CacheIndexView) -> Self {
        match value {
            CacheIndexView::NotCached => CacheIndex(0),
            CacheIndexView::Cached(index) => CacheIndex(index + 1),
        }
    }
}

impl CacheIndex {
    fn view(self) -> CacheIndexView {
        if self.0 == 0 {
            CacheIndexView::NotCached
        } else {
            CacheIndexView::Cached(self.0 - 1)
        }
    }
}

struct Code {
    address_index_range: Range<AddressIndex>,
}

impl Jit {
    pub fn new() -> Self {
        Jit {
            address_cache_index: vec![CacheIndex::from(CacheIndexView::NotCached); ADDRESSES]
                .into_boxed_slice(),
            cache: Vec::new(),
        }
    }

    pub fn code(&mut self, address: MemoryOrScratchpadAddress) -> &Code {
        let address_index = Self::address_index(address);
        match self.address_cache_index[address_index.0 as usize].view() {
            CacheIndexView::NotCached => {
                if self.cache.len() >= MAX_CACHED as usize {
                    self.remove_from_cache(0);
                }
                let code = self.compile_from(address);
                let cache_index = CacheIndex::from(CacheIndexView::Cached(self.cache.len() as u16));
                for i in code.address_index_range.start.0..code.address_index_range.end.0 {
                    self.address_cache_index[i as usize] = cache_index;
                }
                self.cache.push(code);
                &self.cache[self.cache.len() - 1]
            }
            CacheIndexView::Cached(cache_index) => {
                let code = &self.cache[cache_index as usize];
                if code.address_index_range.start == address_index {
                    return &self.cache[cache_index as usize];
                }
                for i in code.address_index_range.start.0..code.address_index_range.end.0 {
                    self.address_cache_index[i as usize] =
                        CacheIndex::from(CacheIndexView::NotCached);
                }
                let new_code = self.compile_from(address);
                for i in new_code.address_index_range.start.0..new_code.address_index_range.end.0 {
                    self.address_cache_index[i as usize] =
                        CacheIndex::from(CacheIndexView::Cached(cache_index));
                }
                let code = &mut self.cache[cache_index as usize];
                *code = new_code;
                code
            }
        }
    }

    fn compile_from(&mut self, address: MemoryOrScratchpadAddress) -> Code {
        todo!()
    }

    fn remove_from_cache(&mut self, cache_index: u16) {
        let old = self.cache.swap_remove(cache_index as usize);
        for i in old.address_index_range.start.0..old.address_index_range.end.0 {
            self.address_cache_index[i as usize] = CacheIndex::from(CacheIndexView::NotCached);
        }
        if let Some(moved) = self.cache.get(cache_index as usize) {
            for i in moved.address_index_range.start.0..moved.address_index_range.end.0 {
                self.address_cache_index[i as usize] =
                    CacheIndex::from(CacheIndexView::Cached(cache_index));
            }
        }
    }

    fn invalidate_address(&mut self, address: MemoryOrScratchpadAddress) {
        let address_index = Self::address_index(address);
        match self.address_cache_index[address_index.0 as usize].view() {
            CacheIndexView::NotCached => {}
            CacheIndexView::Cached(cache_index) => self.remove_from_cache(cache_index),
        }
    }

    fn address_index(address: MemoryOrScratchpadAddress) -> AddressIndex {
        match address.view() {
            MemoryOrScratchpadAddressView::Memory(address) => {
                assert!(address & (std::mem::size_of::<u32>() - 1) as u32 == 0);
                match address {
                    0x0000_0000..0x1000_0000 => {
                        let address = address & (MAIN_MEMORY_SIZE as u32 - 1);
                        AddressIndex(FIRST_MAIN_MEMORY_INDEX + (address / 4))
                    }
                    0x1FC0_0000..0x2000_0000 => {
                        let address = address & (BOOT_MEMORY_SIZE as u32 - 1);
                        AddressIndex(FIRST_BOOT_MEMORY_INDEX + (address / 4))
                    }
                    _ => {
                        panic!("Invalid read at address: 0x{:08x}", address);
                    }
                }
            }
            MemoryOrScratchpadAddressView::Scratchpad(address) => {
                assert!(address & (std::mem::size_of::<u32>() - 1) as u32 == 0);
                let address = address & (SCRATCHPAD_SIZE as u32 - 1);
                AddressIndex(FIRST_SCRATCHPAD_INDEX + (address / 4))
            }
        }
    }
}
