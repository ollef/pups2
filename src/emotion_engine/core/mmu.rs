use std::{fmt::LowerHex, ops::Range};

use enum_map::EnumMap;

use crate::{
    bits::Bits,
    bytes::Bytes,
    emotion_engine::bus::{Bus, PhysicalAddress},
};

use super::{Core, Mode};

const PAGE_BITS: u32 = 20;
const OFFSET_BITS: u32 = 32 - PAGE_BITS;
const PAGE_SIZE: u32 = 1 << OFFSET_BITS;
const OFFSET_MASK: u32 = PAGE_SIZE - 1;
const PAGES: u32 = 1 << PAGE_BITS;

pub struct Mmu {
    tlb_entries: Box<[TlbEntry]>,
    pages: EnumMap<Mode, Box<[PhysicalAddress]>>,
}

pub struct TlbEntry {
    raw: u128,
}

impl Mmu {
    pub fn new() -> Mmu {
        let mut pages = EnumMap::from_fn(|_| {
            vec![PhysicalAddress::memory(0); PAGES as usize].into_boxed_slice()
        });
        let kernel_pages = &mut pages[Mode::Kernel];
        // kseg0 and kseg1 are mapped directly to physical memory.
        for address in (0x8000_0000..0xC000_0000).step_by(PAGE_SIZE as usize) {
            let page = address >> OFFSET_BITS;
            kernel_pages[page as usize] = PhysicalAddress::memory(address & 0x1FFF_FFFF);
        }
        Mmu {
            tlb_entries: (0..48).map(|_| TlbEntry::new()).collect(),
            pages,
        }
    }

    pub fn virtual_to_physical(&self, virtual_address: u32, mode: Mode) -> PhysicalAddress {
        let page = virtual_address >> OFFSET_BITS;
        let physical_frame_start = self.pages[mode][page as usize];
        physical_frame_start + (virtual_address & OFFSET_MASK)
    }

    pub fn physically_consecutive(&self, virtual_range: Range<u32>, mode: Mode) -> bool {
        let start_page = virtual_range.start >> OFFSET_BITS;
        let end_page = (virtual_range.end - 1) >> OFFSET_BITS;
        let mut physical_frame = self.pages[mode][start_page as usize];
        for page in start_page..=end_page {
            if self.pages[mode][page as usize] != physical_frame {
                return false;
            }
            physical_frame += PAGE_SIZE;
        }
        true
    }

    // TODO: This is for testing
    pub fn mmap(&mut self, virtual_address: u32, size: u32, physical_address: u32) {
        assert!(virtual_address & OFFSET_MASK == 0);
        assert!(physical_address & OFFSET_MASK == 0);
        let start_page = virtual_address >> OFFSET_BITS;
        let end_page = (virtual_address + size - 1) >> OFFSET_BITS;
        for page in start_page..=end_page {
            let physical_frame = physical_address + ((page - start_page) << OFFSET_BITS);
            self.pages[Mode::Kernel][page as usize] = PhysicalAddress::memory(physical_frame);
            self.pages[Mode::Supervisor][page as usize] = PhysicalAddress::memory(physical_frame);
            self.pages[Mode::User][page as usize] = PhysicalAddress::memory(physical_frame);
        }
    }
}

impl TlbEntry {
    pub fn new() -> TlbEntry {
        TlbEntry { raw: 0 }
    }

    pub fn mask(&self) -> u16 {
        self.raw.bits(109..=120) as u16
    }

    pub fn virtual_page_number_div_2(&self) -> u32 {
        self.raw.bits(77..=95) as u32
    }

    pub fn global(&self) -> bool {
        self.raw.bit(76)
    }

    pub fn address_space_id(&self) -> u8 {
        self.raw.bits(64..=71) as u8
    }

    pub fn scratchpad(&self) -> bool {
        self.raw.bit(63)
    }

    pub fn page_frame_number_even(&self) -> u32 {
        self.raw.bits(38..=57) as u32
    }

    pub fn cache_mode_even(&self) -> u8 {
        self.raw.bits(35..=37) as u8
    }

    pub fn dirty_even(&self) -> bool {
        self.raw.bit(34)
    }

    pub fn valid_even(&self) -> bool {
        self.raw.bit(33)
    }

    pub fn page_frame_number_odd(&self) -> u32 {
        self.raw.bits(6..=25) as u32
    }

    pub fn cache_mode_odd(&self) -> u8 {
        self.raw.bits(3..=5) as u8
    }

    pub fn dirty_odd(&self) -> bool {
        self.raw.bit(2)
    }

    pub fn valid_odd(&self) -> bool {
        self.raw.bit(1)
    }
}

impl Core {
    pub fn write_virtual<T: Bytes + LowerHex>(&self, bus: &mut Bus, address: u32, value: T) {
        let physical_address = self.mmu.virtual_to_physical(address, self.mode);
        bus.write(physical_address, value);
    }

    pub fn read_virtual<T: Bytes + LowerHex>(&self, bus: &mut Bus, address: u32) -> T {
        let physical_address = self.mmu.virtual_to_physical(address, self.mode);
        bus.read(physical_address)
    }
}
