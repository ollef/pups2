use std::collections::{btree_map::Entry, BTreeMap};

use mmap_rs::{MmapMut, MmapOptions, UnsafeMmapFlags};

#[derive(Debug, Default)]
pub struct ExecutableMemoryAllocator {
    mmaps: Vec<MmapMut>,
    free_by_len: BTreeMap<u32, Vec<*const u8>>,
    free_by_address: BTreeMap<*const u8, u32>,
}

#[derive(Debug, Clone)]
struct Block {
    address: *const u8,
    len: u32,
}

impl Block {
    pub fn end(&self) -> *const u8 {
        unsafe { self.address.add(self.len as usize) }
    }
}

extern "C" {
    /// This function is provided by LLVM to clear the instruction cache for the specified range.
    fn __clear_cache(start: *mut core::ffi::c_void, end: *mut core::ffi::c_void);
}

impl ExecutableMemoryAllocator {
    fn insert_free_block(&mut self, mut block: Block) {
        if block.len == 0 {
            return;
        }
        if let Some((next_block_address, next_block_len)) =
            self.free_by_address.remove_entry(&block.end())
        {
            block.len += next_block_len;
            match self.free_by_len.entry(next_block_len) {
                Entry::Vacant(_) => panic!("Block not found"),
                Entry::Occupied(mut occupied_entry) => {
                    let blocks = occupied_entry.get_mut();
                    blocks.retain(|address| *address != next_block_address);
                    if blocks.is_empty() {
                        occupied_entry.remove();
                    }
                }
            }
        }
        if let Some((prev_block_address, prev_block_len)) =
            self.free_by_address.range_mut(..block.address).next_back()
        {
            if unsafe { prev_block_address.add(*prev_block_len as usize) } == block.address {
                match self.free_by_len.entry(*prev_block_len) {
                    Entry::Vacant(_) => panic!("Block not found"),
                    Entry::Occupied(mut occupied_entry) => {
                        let blocks = occupied_entry.get_mut();
                        blocks.retain(|address| *address != *prev_block_address);
                        if blocks.is_empty() {
                            occupied_entry.remove();
                        }
                    }
                }
                *prev_block_len += block.len;
                self.free_by_len
                    .entry(*prev_block_len)
                    .or_default()
                    .push(*prev_block_address);
                return;
            }
        }
        self.free_by_len
            .entry(block.len)
            .or_default()
            .push(block.address);
        self.free_by_address.insert(block.address, block.len);
    }

    fn try_allocate(&mut self, data: &[u8]) -> Option<*const u8> {
        let len_including_size =
            (data.len().div_ceil(4) * 4) as u32 + std::mem::size_of::<u32>() as u32;
        if let Some((block_len, blocks)) = self.free_by_len.range_mut(len_including_size..).next() {
            let block_len = *block_len;
            let block_address = blocks.pop().unwrap();
            if blocks.is_empty() {
                self.free_by_len.remove(&block_len);
            }
            self.free_by_address.remove(&block_address).unwrap();
            let allocated_block = Block {
                address: block_address,
                len: len_including_size,
            };
            let after_len = unsafe { block_address.add(std::mem::size_of::<u32>()) };
            unsafe {
                *(block_address as *mut u32) = len_including_size;
                std::ptr::copy_nonoverlapping(data.as_ptr(), after_len as *mut u8, data.len());
            }
            unsafe {
                __clear_cache(
                    after_len as *mut std::ffi::c_void,
                    after_len.add(data.len()) as *mut std::ffi::c_void,
                )
            };
            let free_block = Block {
                address: allocated_block.end(),
                len: block_len - allocated_block.len,
            };
            self.insert_free_block(free_block);
            Some(after_len)
        } else {
            None
        }
    }

    pub fn allocate(&mut self, data: &[u8]) -> *const u8 {
        if let Some(address) = self.try_allocate(data) {
            return address;
        }
        let len_including_size = data.len().div_ceil(4) * 4 + std::mem::size_of::<u32>();
        let mmap_len = len_including_size
            .max(
                self.mmaps
                    .last()
                    .map(|mmap| mmap.len() * 2)
                    .unwrap_or_default(),
            )
            .div_ceil(MmapOptions::page_size())
            * MmapOptions::page_size();
        let mmap = unsafe {
            MmapOptions::new(mmap_len)
                .unwrap()
                .with_unsafe_flags(UnsafeMmapFlags::JIT)
                .map_exec_mut()
                .unwrap()
        };
        self.insert_free_block(Block {
            address: mmap.as_ptr(),
            len: mmap.len() as u32,
        });
        self.mmaps.push(mmap);
        self.try_allocate(data).unwrap()
    }

    pub fn free(&mut self, address: *const u8) {
        let block_address = unsafe { address.sub(std::mem::size_of::<u32>()) };
        let len = unsafe { *(block_address as *const u32) };
        self.insert_free_block(Block {
            address: block_address,
            len,
        });
    }
}
