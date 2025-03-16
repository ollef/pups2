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
            let address = address as usize & (MAIN_MEMORY_SIZE - 1);
            Some(u32::from_le_bytes([
                self.main[address],
                self.main[address + 1],
                self.main[address + 2],
                self.main[address + 3],
            ]))
        } else if (0x1FC0_0000..0x2000_0000).contains(&address) {
            let address = address as usize & (BOOT_MEMORY_SIZE - 1);
            Some(u32::from_le_bytes([
                self.boot[address],
                self.boot[address + 1],
                self.boot[address + 2],
                self.boot[address + 3],
            ]))
        } else {
            None
        }
    }

    pub fn write64(&mut self, address: u32, value: u64) {
        if address < 0x1000_0000 {
            let address = address as usize & (MAIN_MEMORY_SIZE - 1);
            for (i, byte) in value.to_le_bytes().iter().enumerate() {
                self.main[address + i] = *byte;
            }
        } else if (0x1FC0_0000..0x2000_0000).contains(&address) {
            let address = address as usize & (BOOT_MEMORY_SIZE - 1);
            for (i, byte) in value.to_le_bytes().iter().enumerate() {
                self.boot[address + i] = *byte;
            }
        }
    }

    pub fn write128(&mut self, address: u32, value: u128) {
        if address < 0x1000_0000 {
            let address = address as usize & (MAIN_MEMORY_SIZE - 1);
            for (i, byte) in value.to_le_bytes().iter().enumerate() {
                self.main[address + i] = *byte;
            }
        } else if (0x1FC0_0000..0x2000_0000).contains(&address) {
            let address = address as usize & (BOOT_MEMORY_SIZE - 1);
            for (i, byte) in value.to_le_bytes().iter().enumerate() {
                self.boot[address + i] = *byte;
            }
        }
    }
}
