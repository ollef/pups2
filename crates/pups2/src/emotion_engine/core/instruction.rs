use std::fmt::Display;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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

    pub fn definitions(&self) -> impl Iterator<Item = Occurrence> {
        self.raw_definitions()
            .into_iter()
            .take_while(|occ| occ.is_some())
            .filter_map(|occ| occ.and_then(|occ| occ.non_zero()))
    }

    pub fn uses(&self) -> impl Iterator<Item = Occurrence> {
        self.raw_uses()
            .into_iter()
            .take_while(|occ| occ.is_some())
            .filter_map(|occ| occ.and_then(|occ| occ.non_zero()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum CacheOperation {
    IXLTG = 0b00000,
    IXLDT = 0b00001,
    BXLBT = 0b00010,
    IXSTG = 0b00100,
    IXSDT = 0b00101,
    BXSBT = 0b00110,
    IXIN = 0b00111,
    BHINBT = 0b01010,
    IHIN = 0b01011,
    BFH = 0b01100,
    IFL = 0b01110,
    DXLTG = 0b10000,
    DXLDT = 0b10001,
    DXSTG = 0b10010,
    DXSDT = 0b10011,
    DXWBIN = 0b10100,
    DXIN = 0b10110,
    DHWBIN = 0b11000,
    DHIN = 0b11010,
    DHWOIN = 0b11100,
}

impl From<u32> for CacheOperation {
    fn from(value: u32) -> Self {
        CacheOperation::from_u32(value & 0b11111).unwrap()
    }
}

impl Display for CacheOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
