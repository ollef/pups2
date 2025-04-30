use crate::bits::SignExtend;

use super::{control, fpu, instruction_gen::Instruction, register::Register};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Occurrence {
    Core(Register),
    Control(control::Register),
    Fpu(fpu::Register),
}

impl Occurrence {
    pub fn non_zero(self) -> Option<Self> {
        match self {
            Occurrence::Core(register) => register.non_zero().map(Occurrence::Core),
            Occurrence::Control(register) => Some(Occurrence::Control(register)),
            Occurrence::Fpu(register) => Some(Occurrence::Fpu(register)),
        }
    }
}

impl From<Register> for Occurrence {
    fn from(register: Register) -> Self {
        Occurrence::Core(register)
    }
}

impl From<control::Register> for Occurrence {
    fn from(register: control::Register) -> Self {
        Occurrence::Control(register)
    }
}

impl From<fpu::Register> for Occurrence {
    fn from(register: fpu::Register) -> Self {
        Occurrence::Fpu(register)
    }
}

impl Instruction {
    pub fn is_nop(&self) -> bool {
        match self {
            Instruction::Unknown => true,
            Instruction::Sll(reg1, reg2, imm) => *imm == 0 && reg1 == reg2,
            Instruction::Addiu(reg1, reg2, imm) => *imm == 0 && reg1 == reg2,
            Instruction::Ori(reg1, reg2, imm) => *imm == 0 && reg1 == reg2,
            _ => false,
        }
    }

    pub fn branch_target(&self, address: u32) -> Option<u32> {
        match self {
            Instruction::Bltz(_, offset)
            | Instruction::Bgez(_, offset)
            | Instruction::Beq(_, _, offset)
            | Instruction::Bne(_, _, offset)
            | Instruction::Blez(_, offset)
            | Instruction::Beql(_, _, offset)
            | Instruction::Bnel(_, _, offset) => Some({
                let offset: u32 = offset.sign_extend();
                address.wrapping_add(4).wrapping_add(offset << 2)
            }),
            Instruction::J(target) | Instruction::Jal(target) => {
                Some((address.wrapping_add(4) & 0xF000_0000).wrapping_add(target << 2))
            }
            _ => None,
        }
    }
}
