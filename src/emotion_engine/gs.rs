use std::fmt::Display;

use super::bus::Bytes;

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
}

impl Gs {
    pub fn new() -> Gs {
        Gs {
            local_memory: vec![0; LOCAL_MEMORY_SIZE].into_boxed_slice(),
        }
    }

    pub fn write<T: Bytes + Display>(&mut self, address: u32, value: T) {
        panic!("GS write {} to address: 0x{:08X}", value, address);
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        panic!("GS read to address: 0x{:08X}", address);
    }
}
