use crate::{bits::Bits, bytes::Bytes};

use super::Gs;

impl Gs {
    fn psmct32_offset(x: u16, y: u16, width: u16) -> u32 {
        let page_x = x / 64;
        let page_y = y / 32;
        let page_index = page_y as u32 * (width as u32 / 64) + page_x as u32;
        let page_size = 8192;
        let page_offset = page_index * page_size;
        let block_x = (x % 64) / 8;
        let block_y = (y % 32) / 8;
        let block_index = block_x.bit(0).then_some(1).unwrap_or_default()
            | block_x.bit(1).then_some(4).unwrap_or_default()
            | block_x.bit(2).then_some(16).unwrap_or_default()
            | block_y.bit(0).then_some(2).unwrap_or_default()
            | block_y.bit(1).then_some(8).unwrap_or_default();
        let block_size = 256;
        let block_offset = block_index * block_size;
        let local_x = x % 8;
        let local_y = y % 8;
        let local_index = local_x.bit(0).then_some(1).unwrap_or_default()
            | local_x.bit(1).then_some(4).unwrap_or_default()
            | local_x.bit(2).then_some(8).unwrap_or_default()
            | local_y.bit(0).then_some(2).unwrap_or_default()
            | local_y.bit(1).then_some(16).unwrap_or_default()
            | local_y.bit(2).then_some(32).unwrap_or_default();
        let pixel_size = 4;
        let local_offset = local_index * pixel_size;
        page_offset + block_offset + local_offset
    }

    pub fn read_psmct32(&self, base_pointer: u32, x: u16, y: u16, width: u16) -> u32 {
        let address = base_pointer + Self::psmct32_offset(x, y, width);
        u32::from_bytes(&self.local_memory[address as usize..address as usize + 4])
    }

    pub fn write_psmct32(&mut self, base_pointer: u32, x: u16, y: u16, width: u16, value: u32) {
        let address = base_pointer + Self::psmct32_offset(x, y, width);
        self.local_memory[address as usize..address as usize + 4]
            .copy_from_slice(&value.to_bytes());
    }
}
