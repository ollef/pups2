use bitvec::{order::Lsb0, slice::BitSlice, view::BitView};
use enum_map::{Enum, EnumMap};

const PAGE_BITS: u32 = 12;
const PAGE_SIZE: u32 = 1 << PAGE_BITS;

#[derive(Enum, Copy, Clone, Debug)]
pub enum Mode {
    Kernel,
    Supervisor,
    User,
}

pub struct Tlb {
    entries: [Entry; 48],
    pages: EnumMap<Mode, [u32; 1 << (32 - PAGE_BITS)]>,
}
pub struct Entry {
    raw: [u32; 4],
}

impl Tlb {
    pub fn new() -> Tlb {
        let mut pages = EnumMap::from_fn(|_| [0; 1 << (32 - PAGE_BITS)]);
        let kernel_pages = &mut pages[Mode::Kernel];
        // kseg0 and kseg1 are mapped directly to physical memory.
        for address in (0x8000_0000..0xC000_0000).step_by(PAGE_SIZE as usize) {
            let page = address >> PAGE_BITS;
            kernel_pages[page as usize] = address & 0x1FFF_FFFF;
        }
        Tlb {
            entries: std::array::from_fn(|_| Entry::new()),
            pages,
        }
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
