pub trait Bytes {
    type Bytes: AsRef<[u8]>;
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(self) -> Self::Bytes;
}

impl Bytes for u8 {
    type Bytes = [u8; 1];

    fn from_bytes(bytes: &[u8]) -> u8 {
        bytes[0]
    }

    fn to_bytes(self) -> [u8; 1] {
        [self]
    }
}

impl Bytes for u16 {
    type Bytes = [u8; 2];

    fn from_bytes(bytes: &[u8]) -> u16 {
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 2] {
        u16::to_le_bytes(self)
    }
}

impl Bytes for u32 {
    type Bytes = [u8; 4];

    fn from_bytes(bytes: &[u8]) -> u32 {
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 4] {
        u32::to_le_bytes(self)
    }
}

impl Bytes for u64 {
    type Bytes = [u8; 8];

    fn from_bytes(bytes: &[u8]) -> u64 {
        u64::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 8] {
        u64::to_le_bytes(self)
    }
}

impl Bytes for u128 {
    type Bytes = [u8; 16];

    fn from_bytes(bytes: &[u8]) -> u128 {
        u128::from_le_bytes(bytes.try_into().unwrap())
    }

    fn to_bytes(self) -> [u8; 16] {
        u128::to_le_bytes(self)
    }
}
