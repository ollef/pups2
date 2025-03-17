use enum_map::{enum_map, Enum, EnumMap};

use super::{
    memory::Memory,
    register::{Cop0Register, Register},
    tlb::Tlb,
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
    pub tlb: Tlb,
    pub main_thread_stack_base: u32, // TODO: This should be in the thread state
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
            tlb: Tlb::new(),
            main_thread_stack_base: 0,
        }
    }

    pub fn read_register32(&self, register: Register) -> u32 {
        let bytes = &self.registers[register].bytes;
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    pub fn read_register64(&self, register: Register) -> u64 {
        let bytes = &self.registers[register].bytes;
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    pub fn read_register128(&self, register: Register) -> u128 {
        u128::from_le_bytes(self.registers[register].bytes)
    }

    pub fn write_register64(&mut self, register: Register, value: u64) {
        if register == Register::Zero {
            return;
        }
        let bytes = &mut self.registers[register].bytes;
        for (i, byte) in value.to_le_bytes().iter().enumerate() {
            bytes[i] = *byte;
        }
    }

    pub fn write_register128(&mut self, register: Register, value: u128) {
        if register == Register::Zero {
            return;
        }
        let bytes = &mut self.registers[register].bytes;
        *bytes = value.to_le_bytes();
    }
}

pub struct RegisterState {
    pub bytes: [u8; 16],
}

impl RegisterState {
    pub fn new() -> Self {
        RegisterState { bytes: [0; 16] }
    }
}
