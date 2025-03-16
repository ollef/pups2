const MAIN_MEMORY_SIZE: usize = 32 * 1024 * 1024;
const BOOT_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Memory {
    pub main: Vec<u8>,
    pub boot: Vec<u8>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            main: vec![0; MAIN_MEMORY_SIZE],
            boot: vec![0; BOOT_MEMORY_SIZE],
        }
    }

    pub fn read32(&self, address: u32) -> Option<u32> {
        if address < 0x1000_0000 {
            let address = address & (MAIN_MEMORY_SIZE as u32 - 1);
            Some(u32::from_le_bytes([
                self.main[(address & 0x1fffff) as usize],
                self.main[((address + 1) & 0x1fffff) as usize],
                self.main[((address + 2) & 0x1fffff) as usize],
                self.main[((address + 3) & 0x1fffff) as usize],
            ]))
        } else if (0x1FC0_0000..0x2000_0000).contains(&address) {
            let address = address & (BOOT_MEMORY_SIZE as u32 - 1);
            Some(u32::from_le_bytes([
                self.boot[(address & 0x3ffff) as usize],
                self.boot[((address + 1) & 0x3ffff) as usize],
                self.boot[((address + 2) & 0x3ffff) as usize],
                self.boot[((address + 3) & 0x3ffff) as usize],
            ]))
        } else {
            None
        }
    }
}
