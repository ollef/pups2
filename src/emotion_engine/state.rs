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

    pub fn get_register16(&self, register: Register) -> u16 {
        self.registers[register].value as u16
    }

    pub fn get_register32(&self, register: Register) -> u32 {
        self.registers[register].value as u32
    }

    pub fn get_register64(&self, register: Register) -> u64 {
        self.registers[register].value as u64
    }

    pub fn get_register128(&self, register: Register) -> u128 {
        self.registers[register].value
    }

    pub fn set_register64(&mut self, register: Register, value: u64) {
        if register == Register::Zero {
            return;
        }
        let value_ref = &mut self.registers[register].value;
        *value_ref &= !(u64::MAX as u128);
        *value_ref |= value as u128;
    }

    pub fn set_register128(&mut self, register: Register, value: u128) {
        if register == Register::Zero {
            return;
        }
        self.registers[register].value = value;
    }
}

pub struct RegisterState {
    pub value: u128,
}

impl RegisterState {
    pub fn new() -> Self {
        RegisterState { value: 0 }
    }
}
