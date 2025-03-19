pub mod disassembler;
pub mod instruction;
pub mod interpreter;
pub mod mmu;
pub mod register;

use enum_map::{enum_map, Enum, EnumMap};

use {
    mmu::Mmu,
    register::{Cop0Register, GetRegister, Register, RegisterState, SetRegister},
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
