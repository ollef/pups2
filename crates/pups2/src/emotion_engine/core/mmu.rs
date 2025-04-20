use std::{fmt::LowerHex, ops::Range};

use bitvec::vec::BitVec;
use enum_map::{enum_map, EnumMap};

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
    mapped: EnumMap<Mode, BitVec<usize>>,
}

#[derive(Clone, Debug)]
pub struct TlbEntry {
    raw: u128,
}

impl Mmu {
    pub fn new() -> Mmu {
        let mut pages = enum_map! { _ =>
            vec![PhysicalAddress::memory(0); PAGES as usize].into_boxed_slice()
        };
        let mut mapped = enum_map! { _ =>
            BitVec::repeat(false, PAGES as usize)
        };
        let kernel_pages = &mut pages[Mode::Kernel];
        let kernel_mapped = &mut mapped[Mode::Kernel];
        // kseg0 and kseg1 are mapped directly to physical memory.
        let kseg_pages = 0x8000_0000 >> OFFSET_BITS..0xC000_0000 >> OFFSET_BITS;
        for page in kseg_pages.clone() {
            let address = page << OFFSET_BITS;
            kernel_pages[page as usize] = PhysicalAddress::memory(address & 0x1FFF_FFFF);
        }
        kernel_mapped[kseg_pages.start as usize..kseg_pages.end as usize].fill(true);
        Mmu {
            tlb_entries: vec![TlbEntry::new(0); 48].into_boxed_slice(),
            pages,
            mapped,
        }
    }

    pub fn virtual_to_physical(&self, virtual_address: u32, mode: Mode) -> PhysicalAddress {
        let page = virtual_address >> OFFSET_BITS;
        if !self.mapped[mode][page as usize] {
            panic!(
                "Virtual address {:#x} not mapped in mode {:?}",
                virtual_address, mode
            );
        }
        let physical_frame_start = unsafe { *self.pages[mode].get_unchecked(page as usize) };
        PhysicalAddress(physical_frame_start.0 | (virtual_address & OFFSET_MASK))
    }

    pub fn physically_consecutive(&self, virtual_range: Range<u32>, mode: Mode) -> bool {
        let start_page = virtual_range.start >> OFFSET_BITS;
        let end_page = (virtual_range.end - 1) >> OFFSET_BITS;
        let mut physical_frame = self.pages[mode][start_page as usize];
        for page in start_page..=end_page {
            if unsafe { *self.pages[mode].get_unchecked(page as usize) } != physical_frame {
                return false;
            }
            physical_frame += PAGE_SIZE;
        }
        true
    }

    pub fn write_index(&mut self, index: u8, entry: TlbEntry) {
        assert!(index < 48);
        println!(
            "Writing TLB index {:#02x} with entry {:#018x}, mask {:#02x}",
            index,
            entry.raw,
            entry.mask()
        );
        println!(
            "Virtual page number: {:#x}, size: {:#x}, scratchpad: {}",
            entry.virtual_page_number_even(),
            entry.len(),
            entry.scratchpad(),
        );
        assert!(entry.address_space_id() == 0);
        self.unmap_entry(self.tlb_entries[index as usize].clone());
        self.tlb_entries[index as usize] = entry.clone();
        self.map_entry(entry);
    }

    fn unmap_entry(&mut self, entry: TlbEntry) {
        let len = entry.len();
        for (virtual_page, _) in entry.mappings() {
            self.unmap(virtual_page, len);
        }
    }

    fn unmap(&mut self, virtual_page: u32, len: u32) {
        let start_page = virtual_page >> OFFSET_BITS;
        let end_page = (virtual_page + len) >> OFFSET_BITS;
        self.mapped[Mode::Kernel][start_page as usize..end_page as usize].fill(false);
        self.mapped[Mode::Supervisor][start_page as usize..end_page as usize].fill(false);
        self.mapped[Mode::User][start_page as usize..end_page as usize].fill(false);
    }

    fn map_entry(&mut self, entry: TlbEntry) {
        let len = entry.len();
        for (virtual_page, physical_frame) in entry.mappings() {
            self.map(virtual_page, physical_frame, len);
        }
    }

    fn map(&mut self, virtual_page: u32, physical_frame: PhysicalAddress, len: u32) {
        let start_page = virtual_page >> OFFSET_BITS;
        let end_page = (virtual_page + len) >> OFFSET_BITS;
        self.mapped[Mode::Kernel][start_page as usize..end_page as usize].fill(true);
        for page in start_page..end_page {
            let virtual_page = page << OFFSET_BITS;
            let physical_frame = physical_frame + ((page - start_page) << OFFSET_BITS);
            println!(
                "Mapping virtual page {:#x} to physical frame {:#x}",
                virtual_page, physical_frame.0
            );
            self.pages[Mode::Kernel][page as usize] = physical_frame;
            match virtual_page {
                0x0000_0000..0x8000_0000 => {
                    self.pages[Mode::Supervisor][page as usize] = physical_frame;
                    self.mapped[Mode::Supervisor].set(page as usize, true);
                    self.pages[Mode::User][page as usize] = physical_frame;
                    self.mapped[Mode::User].set(page as usize, true);
                }
                0xC000_0000..0xE000_0000 => {
                    self.pages[Mode::Supervisor][page as usize] = physical_frame;
                    self.mapped[Mode::Supervisor].set(page as usize, true);
                }
                _ => {}
            }
        }
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
        self.mapped[Mode::Kernel][start_page as usize..=end_page as usize].fill(true);
        self.mapped[Mode::Supervisor][start_page as usize..=end_page as usize].fill(true);
        self.mapped[Mode::User][start_page as usize..=end_page as usize].fill(true);
    }
}

impl TlbEntry {
    pub const MASK: Range<u8> = 109..121;
    pub const VIRTUAL_PAGE_NUMBER_DIV_2: Range<u8> = 77..96;
    pub const GLOBAL: u8 = 76;
    pub const ADDRESS_SPACE_ID: Range<u8> = 64..72;
    pub const SCRATCHPAD: u8 = 63;
    pub const PAGE_FRAME_NUMBER_EVEN: Range<u8> = 38..58;
    pub const CACHE_MODE_EVEN: Range<u8> = 35..38;
    pub const DIRTY_EVEN: u8 = 34;
    pub const VALID_EVEN: u8 = 33;
    pub const PAGE_FRAME_NUMBER_ODD: Range<u8> = 6..26;
    pub const CACHE_MODE_ODD: Range<u8> = 3..6;
    pub const DIRTY_ODD: u8 = 2;
    pub const VALID_ODD: u8 = 1;

    pub fn new(raw: u128) -> TlbEntry {
        TlbEntry { raw }
    }

    pub fn mask(&self) -> u16 {
        self.raw.bits(Self::MASK) as u16
    }

    pub fn virtual_page_number_even(&self) -> u32 {
        self.raw.bits(Self::VIRTUAL_PAGE_NUMBER_DIV_2) as u32 * 2
    }

    pub fn virtual_page_number_odd(&self) -> u32 {
        self.raw.bits(Self::VIRTUAL_PAGE_NUMBER_DIV_2) as u32 * 2 + 1
    }

    pub fn global(&self) -> bool {
        self.raw.bit(Self::GLOBAL)
    }

    pub fn address_space_id(&self) -> u8 {
        self.raw.bits(Self::ADDRESS_SPACE_ID) as u8
    }

    pub fn scratchpad(&self) -> bool {
        self.raw.bit(Self::SCRATCHPAD)
    }

    pub fn page_frame_number_even(&self) -> u32 {
        self.raw.bits(Self::PAGE_FRAME_NUMBER_EVEN) as u32
    }

    pub fn cache_mode_even(&self) -> u8 {
        self.raw.bits(Self::CACHE_MODE_EVEN) as u8
    }

    pub fn dirty_even(&self) -> bool {
        self.raw.bit(Self::DIRTY_EVEN)
    }

    pub fn valid_even(&self) -> bool {
        self.raw.bit(Self::VALID_EVEN)
    }

    pub fn page_frame_number_odd(&self) -> u32 {
        self.raw.bits(Self::PAGE_FRAME_NUMBER_ODD) as u32
    }

    pub fn cache_mode_odd(&self) -> u8 {
        self.raw.bits(Self::CACHE_MODE_ODD) as u8
    }

    pub fn dirty_odd(&self) -> bool {
        self.raw.bit(Self::DIRTY_ODD)
    }

    pub fn valid_odd(&self) -> bool {
        self.raw.bit(Self::VALID_ODD)
    }

    pub fn len(&self) -> u32 {
        if self.scratchpad() {
            return 16 * 1024;
        }
        match self.mask() {
            0b0000_0000_0000 => 4 * 1024,
            0b0000_0000_0011 => 16 * 1024,
            0b0000_0000_1111 => 64 * 1024,
            0b0000_0011_1111 => 256 * 1024,
            0b0000_1111_1111 => 1024 * 1024,
            0b0011_1111_1111 => 4 * 1024 * 1024,
            0b1111_1111_1111 => 16 * 1024 * 1024,
            _ => panic!("Invalid TLB mask: {:#x}", self.mask()),
        }
    }

    pub fn mappings(&self) -> impl Iterator<Item = (u32, PhysicalAddress)> {
        (if self.scratchpad() {
            [
                Some((
                    self.virtual_page_number_even() << OFFSET_BITS,
                    PhysicalAddress::scratchpad(0),
                )),
                None,
            ]
        } else {
            let frame_mask = !(self.len() - 1);
            let even = (self.page_frame_number_even() << OFFSET_BITS) & frame_mask;
            let odd = (self.page_frame_number_odd() << OFFSET_BITS) & frame_mask;
            assert!(!even.bit(31));
            assert!(!odd.bit(31));
            [
                self.valid_even().then(|| {
                    (
                        self.virtual_page_number_even() << OFFSET_BITS,
                        PhysicalAddress::memory(even),
                    )
                }),
                self.valid_odd().then(|| {
                    (
                        self.virtual_page_number_odd() << OFFSET_BITS,
                        PhysicalAddress::memory(odd),
                    )
                }),
            ]
        })
        .into_iter()
        .flatten()
    }
}

impl Core {
    pub fn write_virtual<T: Bytes + LowerHex>(&self, bus: &mut Bus, address: u32, value: T) {
        let physical_address = self.mmu.virtual_to_physical(address, self.mode);
        bus.write(physical_address, value);
    }

    pub fn read_virtual<T: Bytes + LowerHex + Default>(&self, bus: &mut Bus, address: u32) -> T {
        let physical_address = self.mmu.virtual_to_physical(address, self.mode);
        bus.read(physical_address)
    }
}
