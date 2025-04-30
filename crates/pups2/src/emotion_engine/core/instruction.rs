use crate::bits::{Bits, SignExtend};

use super::{
    control, fpu,
    instruction_gen::{let_operands, Opcode},
    register::Register,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub raw: u32,
}

impl Instruction {
    pub fn decode(raw: u32) -> Self {
        let opcode = Opcode::decode(raw);
        Instruction { opcode, raw }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Use {
    Core(Register),
    Control(control::Register),
    Fpu(fpu::Register),
}

impl Use {
    pub fn non_zero(self) -> Option<Self> {
        match self {
            Use::Core(register) => register.non_zero().map(Use::Core),
            Use::Control(register) => Some(Use::Control(register)),
            Use::Fpu(register) => Some(Use::Fpu(register)),
        }
    }
}

impl From<Register> for Use {
    fn from(register: Register) -> Self {
        Use::Core(register)
    }
}

impl From<control::Register> for Use {
    fn from(register: control::Register) -> Self {
        Use::Control(register)
    }
}

impl From<fpu::Register> for Use {
    fn from(register: fpu::Register) -> Self {
        Use::Fpu(register)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Definition {
    Core(Register),
    Control(control::Register),
    Fpu(fpu::Register),
}

impl Definition {
    pub fn non_zero(self) -> Option<Self> {
        match self {
            Definition::Core(register) => register.non_zero().map(Definition::Core),
            Definition::Control(register) => Some(Definition::Control(register)),
            Definition::Fpu(register) => Some(Definition::Fpu(register)),
        }
    }
}

impl From<Register> for Definition {
    fn from(register: Register) -> Self {
        Definition::Core(register)
    }
}

impl From<control::Register> for Definition {
    fn from(register: control::Register) -> Self {
        Definition::Control(register)
    }
}

impl From<fpu::Register> for Definition {
    fn from(register: fpu::Register) -> Self {
        Definition::Fpu(register)
    }
}

macro_rules! opcode_pattern {
    ($opcode:ident, $raw:expr) => {
        Instruction {
            opcode: Opcode::$opcode,
            ..
        }
    };
    (_, $raw:expr) => {
        _
    };
}
pub(super) use opcode_pattern;

macro_rules! case {
    ($scrutinee:expr,
        $($($opcode:tt $($operands:pat_param)?)|+ => $body:expr),* $(,)?
    ) => {
        let scrutinee = $scrutinee;
        match scrutinee {
            $($(opcode_pattern!($opcode, scrutinee.raw) => {
                let_operands!($($operands)?, $opcode, scrutinee.raw);
                $body
            })+)*
        }
    };
}

pub(super) use case;

impl Instruction {
    pub fn is_nop(self) -> bool {
        case! {self,
            Unknown => true,
            Sll(reg1, reg2, imm) => imm == 0 && reg1 == reg2,
            Addiu(reg1, reg2, imm) => imm == 0 && reg1 == reg2,
            Ori(reg1, reg2, imm) => imm == 0 && reg1 == reg2,
            _ => false,
        }
    }

    pub fn is_branch(self) -> bool {
        self.opcode.is_branch()
    }

    pub fn branch_target(self, address: u32) -> Option<u32> {
        case! {self,
            Bltz(_, offset)
            | Bgez(_, offset)
            | Beq(_, _, offset)
            | Bne(_, _, offset)
            | Blez(_, offset)
            | Beql(_, _, offset)
            | Bnel(_, _, offset) => Some({
                let offset: u32 = offset.sign_extend();
                address.wrapping_add(4).wrapping_add(offset << 2)
            }),
            J target | Jal target  => {
                Some((address.wrapping_add(4) & 0xF000_0000).wrapping_add(target << 2))
            },
            _ => None,
        }
    }
}
