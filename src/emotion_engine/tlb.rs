use bitvec::{order::Lsb0, slice::BitSlice, view::BitView};
use enum_map::EnumMap;

use super::state::Mode;

const PAGE_BITS: u32 = 20;
const OFFSET_BITS: u32 = 32 - PAGE_BITS;
const PAGE_SIZE: u32 = 1 << OFFSET_BITS;
const OFFSET_MASK: u32 = PAGE_SIZE - 1;
const PAGES: u32 = 1 << PAGE_BITS;

pub struct Tlb {
    entries: Vec<Entry>,
    pages: EnumMap<Mode, Vec<u32>>,
}

pub struct Entry {
    raw: [u32; 4],
}

impl Tlb {
    pub fn new() -> Tlb {
        let mut pages = EnumMap::from_fn(|_| vec![0; PAGES as usize]);
        let kernel_pages = &mut pages[Mode::Kernel];
        // kseg0 and kseg1 are mapped directly to physical memory.
        for address in (0x8000_0000..0xC000_0000).step_by(PAGE_SIZE as usize) {
            let page = address >> OFFSET_BITS;
            kernel_pages[page as usize] = address & 0x1FFF_FFFF;
        }
        Tlb {
            entries: (0..48).map(|_| Entry::new()).collect(),
            pages,
        }
    }

    pub fn virtual_to_physical(&self, virtual_address: u32, mode: Mode) -> u32 {
        let page = virtual_address >> OFFSET_BITS;
        let physical_frame_start = self.pages[mode][page as usize];
        physical_frame_start + (virtual_address & OFFSET_MASK)
    }
}

impl Entry {
    pub fn new() -> Entry {
        Entry { raw: [0; 4] }
    }

    pub fn mask(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[109..=120]
    }

    pub fn virtual_page_number_div_2(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[77..=95]
    }

    pub fn global(&self) -> bool {
        self.raw.view_bits::<Lsb0>()[76]
    }

    pub fn address_space_id(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[64..=71]
    }

    pub fn scratchpad(&self) -> bool {
        self.raw.view_bits::<Lsb0>()[63]
    }

    pub fn page_frame_number_even(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[38..=57]
    }

    pub fn cache_mode_even(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[35..=37]
    }

    pub fn dirty_even(&self) -> bool {
        self.raw.view_bits::<Lsb0>()[34]
    }

    pub fn valid_even(&self) -> bool {
        self.raw.view_bits::<Lsb0>()[33]
    }

    pub fn page_frame_number_odd(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[6..=25]
    }

    pub fn cache_mode_odd(&self) -> &BitSlice<u32> {
        &self.raw.view_bits()[3..=5]
    }

    pub fn dirty_odd(&self) -> bool {
        self.raw.view_bits::<Lsb0>()[2]
    }

    pub fn valid_odd(&self) -> bool {
        self.raw.view_bits::<Lsb0>()[1]
    }
}
