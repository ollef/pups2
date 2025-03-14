use enum_map::{enum_map, EnumMap};

use super::register::Register;

pub struct State {
    registers: EnumMap<Register, RegisterState>,
}

impl State {
    pub fn new() -> Self {
        State {
            registers: enum_map! { _ => RegisterState::new() },
        }
    }
}

pub struct RegisterState {
    pub bytes: [u8; 16],
}

impl RegisterState {
    pub fn new() -> Self {
        RegisterState { bytes: [0; 16] }
    }

    pub fn read_u32(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn write_u32(&mut self, value: u32) {
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            self.bytes[i] = *byte;
        }
    }
}
