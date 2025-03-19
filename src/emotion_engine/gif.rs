use std::fmt::Display;

use crate::fifo::Fifo;

use super::bus::Bytes;

pub struct Gif {
    fifo: Fifo<u128>,
}

impl Gif {
    pub fn new() -> Gif {
        Gif {
            fifo: Fifo::with_capacity(16),
        }
    }

    pub fn write<T: Bytes + Display>(&mut self, address: u32, value: T) {
        panic!("GIF write {} to address: 0x{:08X}", value, address);
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        panic!("GIF read to address: 0x{:08X}", address);
    }
}
