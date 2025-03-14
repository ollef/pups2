use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)]
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
    Gp,
    Sp,
    Fp,
    Ra,
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        let value = value & 0b11111;
        unsafe { std::mem::transmute(value as u8) }
    }
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
            Register::Gp => write!(f, "gp"),
            Register::Sp => write!(f, "sp"),
            Register::Fp => write!(f, "fp"),
            Register::Ra => write!(f, "ra"),
        }
    }
}

impl Register {
    fn non_zero(self) -> Option<Self> {
        match self {
            Register::Zero => None,
            _ => Some(self),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Sll(Register, Register, u8),
    Bgez(Register, u16),
    J(u32),
    Jal(u32),
    Beq(Register, Register, u16),
    Bne(Register, Register, u16),
    Addiu(Register, Register, u16),
    Andi(Register, Register, u16),
    Ori(Register, Register, u16),
    Lui(Register, u16),
    Ei,
    Sq(Register, Register, u16),
    Lh(Register, Register, u16),
    Lw(Register, Register, u16),
    Lbu(Register, Register, u16),
    Lwr(Register, Register, u16),
    Sb(Register, Register, u16),
    Sh(Register, Register, u16),
    Sw(Register, Register, u16),
    Ld(Register, Register, u16),
    Sd(Register, Register, u16),
}

impl Instruction {
    fn definitions(&self) -> impl Iterator<Item = Register> {
        (match self {
            Instruction::Sll(a, _, _) => a.non_zero(),
            Instruction::Bgez(_, _) => None,
            Instruction::J(_) => None,
            Instruction::Jal(_) => Some(Register::Ra),
            Instruction::Beq(_, _, _) => None,
            Instruction::Bne(_, _, _) => None,
            Instruction::Addiu(a, _, _) => a.non_zero(),
            Instruction::Andi(a, _, _) => a.non_zero(),
            Instruction::Ori(a, _, _) => a.non_zero(),
            Instruction::Lui(a, _) => a.non_zero(),
            Instruction::Ei => None,
            Instruction::Sq(_, _, _) => None,
            Instruction::Lh(a, _, _) => a.non_zero(),
            Instruction::Lw(a, _, _) => a.non_zero(),
            Instruction::Lbu(a, _, _) => a.non_zero(),
            Instruction::Lwr(a, _, _) => a.non_zero(),
            Instruction::Sb(_, _, _) => None,
            Instruction::Sh(_, _, _) => None,
            Instruction::Sw(_, _, _) => None,
            Instruction::Ld(a, _, _) => a.non_zero(),
            Instruction::Sd(_, _, _) => None,
        })
        .into_iter()
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Sll(a, b, c) => write!(f, "sll {a}, {b}, {c}"),
            Instruction::Bgez(a, b) => write!(f, "bgez {a}, {b:#x}"),
            Instruction::J(a) => write!(f, "j {a:#x}"),
            Instruction::Jal(a) => write!(f, "jal {a:#x}"),
            Instruction::Beq(a, b, c) => write!(f, "beq {a}, {b}, {c:#x}"),
            Instruction::Bne(a, b, c) => write!(f, "bne {a}, {b}, {c:#x}"),
            Instruction::Addiu(a, b, c) => write!(f, "addiu {a}, {b}, {c:#x}"),
            Instruction::Andi(a, b, c) => write!(f, "andi {a}, {b}, {c:#x}"),
            Instruction::Ori(a, b, c) => write!(f, "ori {a}, {b}, {c:#x}"),
            Instruction::Lui(a, b) => write!(f, "lui {a}, {b}"),
            Instruction::Ei => write!(f, "ei"),
            Instruction::Sq(a, b, c) => write!(f, "sq {a}, {c:#x}({b})"),
            Instruction::Lh(a, b, c) => write!(f, "lh {a}, {c:#x}({b})"),
            Instruction::Lw(a, b, c) => write!(f, "lw {a}, {c:#x}({b})"),
            Instruction::Lbu(a, b, c) => write!(f, "lbu {a}, {c:#x}({b})"),
            Instruction::Lwr(a, b, c) => write!(f, "lwr {a}, {c:#x}({b})"),
            Instruction::Sb(a, b, c) => write!(f, "sb {a}, {c:#x}({b})"),
            Instruction::Sh(a, b, c) => write!(f, "sh {a}, {c:#x}({b})"),
            Instruction::Sw(a, b, c) => write!(f, "sw {a}, {c:#x}({b})"),
            Instruction::Ld(a, b, c) => write!(f, "ld {a}, {c:#x}({b})"),
            Instruction::Sd(a, b, c) => write!(f, "sd {a}, {c:#x}({b})"),
        }
    }
}

impl Instruction {
    pub fn is_nop(&self) -> bool {
        match self {
            Instruction::Sll(reg1, reg2, 0) => reg1 == reg2,
            Instruction::Addiu(reg1, reg2, 0) => reg1 == reg2,
            Instruction::Ori(reg1, reg2, 0) => reg1 == reg2,
            _ => false,
        }
    }
}

pub fn disassemble(data: u32) -> Instruction {
    let opcode = data >> 26;
    let s = (data >> 21) & 0b11111;
    let t = (data >> 16) & 0b11111;
    let d = (data >> 11) & 0b11111;
    let rs = Register::from(s);
    let rt = Register::from(t);
    let rd = Register::from(d);
    let shamt = ((data >> 6) & 0b1111) as u8;
    let imm16 = (data & 0b11111111_11111111) as u16;
    let imm26 = data & 0b00000011_11111111_11111111_11111111;
    match opcode {
        0b000000 => Instruction::Sll(rd, rt, shamt),
        0b000001 => match t {
            0b00001 => Instruction::Bgez(rs, imm16),
            _ => panic!("Branch not implemented {:#034b}", data),
        },
        0b000010 => Instruction::J(imm26),
        0b000011 => Instruction::Jal(imm26),
        0b000100 => Instruction::Beq(rs, rt, imm16),
        0b000101 => Instruction::Bne(rs, rt, imm16),
        0b001001 => Instruction::Addiu(rt, rs, imm16),
        0b001100 => Instruction::Andi(rt, rs, imm16),
        0b001101 => Instruction::Ori(rt, rs, imm16),
        0b001111 => Instruction::Lui(rt, imm16),
        0b010000 => match s {
            0b10000 => match data & 0b111111 {
                0b111000 => Instruction::Ei,
                _ => panic!("TLB/Exception not implemented {:#034b}", data),
            },
            _ => panic!("COP0 not implemented {:#034b}", data),
        },
        0b011111 => Instruction::Sq(rt, rs, imm16),
        0b100001 => Instruction::Lh(rt, rs, imm16),
        0b100011 => Instruction::Lw(rt, rs, imm16),
        0b100100 => Instruction::Lbu(rt, rs, imm16),
        0b100101 => Instruction::Lwr(rt, rs, imm16),
        0b101000 => Instruction::Sb(rt, rs, imm16),
        0b101001 => Instruction::Sh(rt, rs, imm16),
        0b101011 => Instruction::Sw(rt, rs, imm16),
        0b110111 => Instruction::Ld(rt, rs, imm16),
        0b111111 => Instruction::Sd(rt, rs, imm16),
        _ => panic!("Not implemented {:#034b}", data),
    }
}
