pub mod control;
pub mod execution;
pub mod fpu;
pub mod instruction;
pub mod instruction_gen;
pub mod interpreter;
pub mod jit;
pub mod mmu;
pub mod register;

use control::Control;
use enum_map::{enum_map, Enum, EnumMap};
use fpu::Fpu;
use jit::Jit;
use register::{GetUpper, SetUpper};

use {
    mmu::Mmu,
    register::{GetRegister, Register, SetRegister},
};

#[derive(Enum, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Mode {
    Kernel,
    Supervisor,
    User,
}

pub struct Core {
    pub state: State,
    pub mode: Mode,
    pub mmu: Mmu,
    pub main_thread_stack_pointer: u32, // TODO: This should be in the thread state
    pub jit: Jit,
}

#[derive(Debug)]
pub struct State {
    pub program_counter: u32,
    pub registers: EnumMap<Register, u128>,
    pub control: Control,
    pub fpu: Fpu,
    pub delayed_branch_target: Option<u32>,
}

impl Core {
    pub fn new() -> Self {
        Core {
            mode: Mode::Kernel,
            state: State {
                program_counter: 0xBFC00000,
                registers: enum_map! { _ => 0 },
                control: Control::new(),
                fpu: Fpu::new(),
                delayed_branch_target: None,
            },
            mmu: Mmu::new(),
            main_thread_stack_pointer: 0,
            jit: Jit::new(),
        }
    }

    #[inline(always)]
    pub fn get_register<T>(&self, register: Register) -> T
    where
        u128: GetRegister<T>,
    {
        self.state.registers[register].get_register()
    }

    #[inline(always)]
    pub fn set_register<T>(&mut self, register: Register, value: T)
    where
        u128: SetRegister<T>,
    {
        if register == Register::Zero {
            return;
        }
        self.state.registers[register].set_register(value);
    }

    #[inline(always)]
    pub fn get_upper<T>(&self, register: Register) -> T
    where
        u128: GetUpper<T>,
    {
        self.state.registers[register].get_upper()
    }

    #[inline(always)]
    pub fn set_upper<T>(&mut self, register: Register, value: T)
    where
        u128: SetUpper<T>,
    {
        if register == Register::Zero {
            return;
        }
        self.state.registers[register].set_upper(value);
    }
}
