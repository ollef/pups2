use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    Zero,
    At,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    GP,
    SP,
    FP,
    RA,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Zero => write!(f, "zero"),
            Register::At => write!(f, "at"),
            Register::V0 => write!(f, "v0"),
            Register::V1 => write!(f, "v1"),
            Register::A0 => write!(f, "a0"),
            Register::A1 => write!(f, "a1"),
            Register::A2 => write!(f, "a2"),
            Register::A3 => write!(f, "a3"),
            Register::T0 => write!(f, "t0"),
            Register::T1 => write!(f, "t1"),
            Register::T2 => write!(f, "t2"),
            Register::T3 => write!(f, "t3"),
            Register::T4 => write!(f, "t4"),
            Register::T5 => write!(f, "t5"),
            Register::T6 => write!(f, "t6"),
            Register::T7 => write!(f, "t7"),
            Register::S0 => write!(f, "s0"),
            Register::S1 => write!(f, "s1"),
            Register::S2 => write!(f, "s2"),
            Register::S3 => write!(f, "s3"),
            Register::S4 => write!(f, "s4"),
            Register::S5 => write!(f, "s5"),
            Register::S6 => write!(f, "s6"),
            Register::S7 => write!(f, "s7"),
            Register::T8 => write!(f, "t8"),
            Register::T9 => write!(f, "t9"),
            Register::K0 => write!(f, "k0"),
            Register::K1 => write!(f, "k1"),
            Register::GP => write!(f, "gp"),
            Register::SP => write!(f, "sp"),
            Register::FP => write!(f, "fp"),
            Register::RA => write!(f, "ra"),
        }
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        let value = value & 0b11111;
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[derive(Debug, derive_more::Display)]
pub enum Instruction {
    #[display("lui {}, {}", _0, _1)]
    Lui(Register, u16),
    #[display("addiu {}, {}, {:#x}", _0, _1, _2)]
    Addiu(Register, Register, u16),
    #[display("sll {}, {}, {}", _0, _1, _2)]
    Sll(Register, Register, u8),
    #[display("sq {}, {:#x}({})", _0, _1, _2)]
    Sq(Register, u16, Register),
    #[display("bne {}, {}, {:#x}", _0, _1, _2)]
    Bne(Register, Register, u16),
    #[display("jal {:#x}", _0)]
    Jal(u32),
}

impl Instruction {
    pub fn is_nop(&self) -> bool {
        match self {
            Instruction::Sll(reg1, reg2, 0) => reg1 == reg2,
            Instruction::Addiu(reg1, reg2, 0) => reg1 == reg2,
            _ => false,
        }
    }
}

pub fn disassemble(data: u32) -> Instruction {
    let opcode = data >> 26;
    let rs = Register::from(data >> 21);
    let rt = Register::from(data >> 16);
    let rd = Register::from(data >> 11);
    let shamt = (data >> 6) as u8;
    let imm16 = (data & 0b11111111_11111111) as u16;
    let imm26 = data & 0b00000011_11111111_11111111_11111111;
    match opcode {
        0b001111 => Instruction::Lui(rt, imm16),
        0b001001 => Instruction::Addiu(rt, rs, imm16),
        0b000000 => Instruction::Sll(rd, rt, shamt),
        0b011111 => Instruction::Sq(rt, imm16, rs),
        0b000101 => Instruction::Bne(rs, rt, imm16),
        0b000011 => Instruction::Jal(imm26),
        _ => panic!("Not implemented {:#034b}", data),
    }
}
