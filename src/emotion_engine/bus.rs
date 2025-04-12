use std::fmt::LowerHex;

use crate::{bits::Bits, bytes::Bytes};

use super::{dmac::Dmac, gif::Gif, gs::Gs, timer::Timer};

pub const MAIN_MEMORY_SIZE: usize = 32 * 1024 * 1024;
pub const BOOT_MEMORY_SIZE: usize = 4 * 1024 * 1024;
pub const SCRATCHPAD_SIZE: usize = 16 * 1024;

pub struct Bus {
    pub main_memory: Box<[u8]>,
    pub boot_memory: Box<[u8]>,
    pub scratchpad: Box<[u8]>,
    pub timer: Timer,
    pub gif: Gif,
    pub dmac: Dmac,
    pub gs: Gs,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicalAddress(pub u32);

pub enum PhysicalAddressView {
    Memory(u32),
    Scratchpad(u32),
}

impl From<PhysicalAddressView> for PhysicalAddress {
    fn from(value: PhysicalAddressView) -> Self {
        match value {
            PhysicalAddressView::Memory(address) => PhysicalAddress(address),
            PhysicalAddressView::Scratchpad(address) => {
                PhysicalAddress(address | u32::mask(31..=31))
            }
        }
    }
}

impl PhysicalAddress {
    pub fn view(&self) -> PhysicalAddressView {
        if self.0.bit(31) {
            PhysicalAddressView::Scratchpad(self.0.bits(0..31))
        } else {
            PhysicalAddressView::Memory(self.0.bits(0..31))
        }
    }
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
            boot_memory: vec![0; BOOT_MEMORY_SIZE].into_boxed_slice(),
            scratchpad: vec![0; SCRATCHPAD_SIZE].into_boxed_slice(),
            timer: Timer::new(),
            gif: Gif::new(),
            dmac: Dmac::default(),
            gs: Gs::new(),
        }
    }

    pub fn read_memory_or_scratchpad<T: Bytes + LowerHex>(&self, address: PhysicalAddress) -> T {
        match address.view() {
            PhysicalAddressView::Memory(address) => self.read(address),
            PhysicalAddressView::Scratchpad(address) => self.read_scratchpad(address),
        }
    }

    pub fn read<T: Bytes + LowerHex>(&self, address: u32) -> T {
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        match address {
            0x0000_0000..0x1000_0000 => {
                let address = address as usize & (MAIN_MEMORY_SIZE - 1);
                T::from_bytes(&self.main_memory[address..address + std::mem::size_of::<T>()])
            }
            0x1000_0000..0x1000_2000 => {
                let result = self.timer.read(address);
                println!("Read from TIMER: 0x{:08x}==0x{:08x}", address, result);
                result
            }
            0x1000_3000..0x1000_3800 => {
                let result = self.gif.read(address);
                println!("Read from GIF: 0x{:08x}==0x{:08x}", address, result);
                result
            }
            0x1000_8000..0x1000_F000 => {
                let result = self.dmac.read(address);
                // println!("Read from DMAC: 0x{:08x}==0x{:08x}", address, result);
                result
            }
            0x1200_0000..0x1201_0000 => {
                let result = self.gs.read_privileged(address);
                // println!("Read from GS: 0x{:08x}==0x{:08x}", address, result);
                result
            }
            0x1FC0_0000..0x2000_0000 => {
                let address = address as usize & (BOOT_MEMORY_SIZE - 1);
                let result =
                    T::from_bytes(&self.boot_memory[address..address + std::mem::size_of::<T>()]);
                println!("Read from Boot memory: 0x{:08x}==0x{:08x}", address, result);
                result
            }
            _ => {
                panic!("Invalid read at address: 0x{:08x}", address);
            }
        }
    }

    pub fn read_scratchpad<T: Bytes + LowerHex>(&self, address: u32) -> T {
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        let address = address as usize & (SCRATCHPAD_SIZE - 1);
        T::from_bytes(&self.scratchpad[address..address + std::mem::size_of::<T>()])
    }

    pub fn write<T: Bytes + LowerHex>(&mut self, address: u32, value: T) {
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        match address {
            0x0000_0000..0x1000_0000 => {
                let address = address as usize & (MAIN_MEMORY_SIZE - 1);
                // println!("Write to main memory: 0x{:08x}:=0x{:08x}", address, value);
                self.main_memory[address..address + std::mem::size_of::<T>()]
                    .copy_from_slice(value.to_bytes().as_ref());
            }
            0x1000_0000..0x1000_2000 => {
                println!("Write to TIMER: 0x{:08x}:=0x{:08x}", address, value);
                self.timer.write(address, value)
            }
            0x1000_3000..0x1000_3800 => {
                println!("Write to GIF: 0x{:08x}:=0x{:08x}", address, value);
                self.gif.write(address, value)
            }
            0x1000_8000..0x1000_F000 => {
                // println!("Write to DMAC: 0x{:08x}:=0x{:08x}", address, value);
                self.dmac.write(address, value)
            }
            0x1200_0000..0x1201_0000 => {
                println!("Write to GS: 0x{:08x}:=0x{:08x}", address, value);
                self.gs.write_privileged(address, value)
            }
            0x1FC0_0000..0x2000_0000 => {
                let address = address as usize & (BOOT_MEMORY_SIZE - 1);
                println!("Write to boot memory: 0x{:08x}:=0x{:08x}", address, value);
                self.main_memory[address..address + std::mem::size_of::<T>()]
                    .copy_from_slice(value.to_bytes().as_ref());
            }
            _ => {
                panic!("Invalid write at address: 0x{:08x}", address);
            }
        }
    }

    pub fn write_scratchpad<T: Bytes + LowerHex>(&mut self, address: u32, value: T) {
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        let address = address as usize & (SCRATCHPAD_SIZE - 1);
        self.scratchpad[address..address + std::mem::size_of::<T>()]
            .copy_from_slice(value.to_bytes().as_ref());
    }

    pub fn write_memory_or_scratchpad<T: Bytes + LowerHex>(
        &mut self,
        address: PhysicalAddress,
        value: T,
    ) {
        match address.view() {
            PhysicalAddressView::Memory(address) => self.write(address, value),
            PhysicalAddressView::Scratchpad(address) => self.write_scratchpad(address, value),
        }
    }
}
