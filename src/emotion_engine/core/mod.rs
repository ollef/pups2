pub mod disassembler;
pub mod instruction;
pub mod interpreter;
pub mod mmu;
pub mod register;

use enum_map::{enum_map, Enum, EnumMap};

use {
    mmu::Mmu,
    register::{ControlRegister, GetRegister, Register, SetRegister},
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
    pub registers: EnumMap<Register, u128>,
    pub cop0_registers: EnumMap<ControlRegister, u32>,
    pub delayed_branch_target: Option<u32>,
    pub mmu: Mmu,
    pub main_thread_stack_pointer: u32, // TODO: This should be in the thread state
}

impl Core {
    pub fn new(program_counter: u32) -> Self {
        Core {
            mode: Mode::Kernel,
            program_counter,
            registers: enum_map! { _ => 0 },
            cop0_registers: enum_map! { _ => 0 },
            delayed_branch_target: None,
            mmu: Mmu::new(),
            main_thread_stack_pointer: 0,
        }
    }

    pub fn get_register<T>(&self, register: Register) -> T
    where
        u128: GetRegister<T>,
    {
        self.registers[register].get_register()
    }

    pub fn set_register<T>(&mut self, register: Register, value: T)
    where
        u128: SetRegister<T>,
    {
        if register == Register::Zero {
            return;
        }
        self.registers[register].set_register(value);
    }
}
