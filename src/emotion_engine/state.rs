use enum_map::{enum_map, Enum, EnumMap};

use super::{
    memory::Memory,
    register::{Cop0Register, Register},
};

#[derive(Enum, Copy, Clone, Debug)]
pub enum Mode {
    Kernel,
    Supervisor,
    User,
}

pub struct State {
    pub mode: Mode,
    pub program_counter: u32,
    pub registers: EnumMap<Register, RegisterState>,
    pub cop0_registers: EnumMap<Cop0Register, u32>,
    pub delayed_branch_target: Option<u32>,
    pub memory: Memory,
}

impl State {
    pub fn new(program_counter: u32) -> Self {
        State {
            mode: Mode::Kernel,
            program_counter,
            registers: enum_map! { _ => RegisterState::new() },
            cop0_registers: enum_map! { _ => 0 },
            delayed_branch_target: None,
            memory: Memory::new(),
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

    pub fn read32(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn read64(&self) -> u64 {
        u64::from_le_bytes([
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5],
            self.bytes[6],
            self.bytes[7],
        ])
    }

    pub fn write32(&mut self, value: u32) {
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            self.bytes[i] = *byte;
        }
    }

    pub fn write64(&mut self, value: u64) {
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            self.bytes[i] = *byte;
        }
    }
}
