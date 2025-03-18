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

    pub fn read<T: Bytes>(&self, address: u32) -> Option<T> {
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        match address {
            0x0000_0000..0x1000_0000 => {
                let address = address as usize & (MAIN_MEMORY_SIZE - 1);
                Some(T::from_bytes(
                    &self.main[address..address + std::mem::size_of::<T>()],
                ))
            }
            0x1FC0_0000..0x2000_0000 => {
                let address = address as usize & (BOOT_MEMORY_SIZE - 1);
                Some(T::from_bytes(
                    &self.boot[address..address + std::mem::size_of::<T>()],
                ))
            }
            _ => None,
        }
    }

    pub fn write<T: Bytes>(&mut self, address: u32, value: T) {
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        match address {
            0x0000_0000..0x1000_0000 => {
                let address = address as usize & (MAIN_MEMORY_SIZE - 1);
                self.main[address..address + std::mem::size_of::<T>()]
                    .copy_from_slice(value.to_bytes().as_ref());
            }
            0x1FC0_0000..0x2000_0000 => {
                let address = address as usize & (BOOT_MEMORY_SIZE - 1);
                self.main[address..address + std::mem::size_of::<T>()]
                    .copy_from_slice(value.to_bytes().as_ref());
            }
            _ => {
                panic!("Invalid address: 0x{:08X}", address);
            }
        }
    }
}

pub trait Bytes {
    type ToBytes: AsRef<[u8]>;
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(self) -> Self::ToBytes;
}

impl Bytes for u8 {
    type ToBytes = [u8; 1];

    fn from_bytes(bytes: &[u8]) -> u8 {
        bytes[0]
    }

    fn to_bytes(self) -> [u8; 1] {
        [self]
    }
}

impl Bytes for u16 {
    type ToBytes = [u8; 2];

    fn from_bytes(bytes: &[u8]) -> u16 {
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 2] {
        u16::to_le_bytes(self)
    }
}

impl Bytes for u32 {
    type ToBytes = [u8; 4];

    fn from_bytes(bytes: &[u8]) -> u32 {
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 4] {
        u32::to_le_bytes(self)
    }
}

impl Bytes for u64 {
    type ToBytes = [u8; 8];

    fn from_bytes(bytes: &[u8]) -> u64 {
        u64::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 8] {
        u64::to_le_bytes(self)
    }
}

impl Bytes for u128 {
    type ToBytes = [u8; 16];

    fn from_bytes(bytes: &[u8]) -> u128 {
        u128::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 16] {
        u128::to_le_bytes(self)
    }
}
