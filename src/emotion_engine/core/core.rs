use enum_map::{enum_map, Enum, EnumMap};

use super::{
    mmu::Mmu,
    register::{Cop0Register, Register},
};

#[derive(Enum, Copy, Clone, Debug)]
pub enum Mode {
    Kernel,
    Supervisor,
    User,
}

pub struct Core {
    pub mode: Mode,
    pub program_counter: u32,
    pub registers: EnumMap<Register, RegisterState>,
    pub cop0_registers: EnumMap<Cop0Register, u32>,
    pub delayed_branch_target: Option<u32>,
    pub mmu: Mmu,
    pub main_thread_stack_base: u32, // TODO: This should be in the thread state
}

impl Core {
    pub fn new(program_counter: u32) -> Self {
        Core {
            mode: Mode::Kernel,
            program_counter,
            registers: enum_map! { _ => RegisterState::new() },
            cop0_registers: enum_map! { _ => 0 },
            delayed_branch_target: None,
            mmu: Mmu::new(),
            main_thread_stack_base: 0,
        }
    }

    pub fn get_register<T>(&self, register: Register) -> T
    where
        RegisterState: GetRegister<T>,
    {
        self.registers[register].get_register()
    }

    pub fn set_register<T>(&mut self, register: Register, value: T)
    where
        RegisterState: SetRegister<T>,
    {
        if register == Register::Zero {
            return;
        }
        self.registers[register].set_register(value);
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

pub trait SetRegister<T> {
    fn set_register(&mut self, value: T);
}

pub trait GetRegister<T> {
    fn get_register(&self) -> T;
}

impl SetRegister<u64> for RegisterState {
    fn set_register(&mut self, value: u64) {
        self.value = value as u128 | (self.value & 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000);
    }
}

impl SetRegister<u128> for RegisterState {
    fn set_register(&mut self, value: u128) {
        self.value = value;
    }
}

impl GetRegister<u16> for RegisterState {
    fn get_register(&self) -> u16 {
        self.value as u16
    }
}

impl GetRegister<u32> for RegisterState {
    fn get_register(&self) -> u32 {
        self.value as u32
    }
}

impl GetRegister<u64> for RegisterState {
    fn get_register(&self) -> u64 {
        self.value as u64
    }
}

impl GetRegister<u128> for RegisterState {
    fn get_register(&self) -> u128 {
        self.value
    }
}
