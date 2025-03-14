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
    Unknown,
    Sll(Register, Register, u8),
    Srl(Register, Register, u8),
    Sra(Register, Register, u8),
    Sllv(Register, Register, Register),
    Srav(Register, Register, Register),
    Jr(Register),
    Jalr(Register, Register),
    Movz(Register, Register, Register),
    Syscall,
    Sync,
    Mfhi(Register),
    Mthi(Register),
    Mflo(Register),
    Mult(Register, Register, Register),
    Add(Register, Register, Register),
    Addu(Register, Register, Register),
    And(Register, Register, Register),
    Or(Register, Register, Register),
    Mtsa(Register),
    Slt(Register, Register, Register),
    Sltu(Register, Register, Register),
    Daddu(Register, Register, Register),
    Dsub(Register, Register, Register),
    Dsubu(Register, Register, Register),
    Tge(Register, Register),
    Tgeu(Register, Register),
    Tlt(Register, Register),
    Tltu(Register, Register),
    Teq(Register, Register),
    Dsll(Register, Register, u8),
    Dsrl(Register, Register, u8),
    Dsra(Register, Register, u8),
    Dsll32(Register, Register, u8),
    Dsrl32(Register, Register, u8),
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
            Instruction::Unknown => None,
            Instruction::Sll(a, _, _) => Some(*a),
            Instruction::Srl(a, _, _) => Some(*a),
            Instruction::Sra(a, _, _) => Some(*a),
            Instruction::Sllv(a, _, _) => Some(*a),
            Instruction::Srav(a, _, _) => Some(*a),
            Instruction::Jr(_) => None,
            Instruction::Jalr(a, _) => Some(*a),
            Instruction::Movz(a, _, _) => Some(*a),
            Instruction::Syscall => None,
            Instruction::Sync => None,
            Instruction::Mfhi(a) => Some(*a),
            Instruction::Mthi(_) => None,
            Instruction::Mflo(a) => Some(*a),
            Instruction::Mult(a, _, _) => Some(*a),
            Instruction::Add(a, _, _) => Some(*a),
            Instruction::Addu(a, _, _) => Some(*a),
            Instruction::And(a, _, _) => Some(*a),
            Instruction::Or(a, _, _) => Some(*a),
            Instruction::Mtsa(_) => None, // TODO: check
            Instruction::Slt(a, _, _) => Some(*a),
            Instruction::Sltu(a, _, _) => Some(*a),
            Instruction::Daddu(a, _, _) => Some(*a),
            Instruction::Dsub(a, _, _) => Some(*a),
            Instruction::Dsubu(a, _, _) => Some(*a),
            Instruction::Tge(_, _) => None,
            Instruction::Tgeu(_, _) => None,
            Instruction::Tlt(_, _) => None,
            Instruction::Tltu(_, _) => None,
            Instruction::Teq(_, _) => None,
            Instruction::Dsll(a, _, _) => Some(*a),
            Instruction::Dsrl(a, _, _) => Some(*a),
            Instruction::Dsra(a, _, _) => Some(*a),
            Instruction::Dsll32(a, _, _) => Some(*a),
            Instruction::Dsrl32(a, _, _) => Some(*a),
            Instruction::Bgez(_, _) => None,
            Instruction::J(_) => None,
            Instruction::Jal(_) => Some(Register::Ra),
            Instruction::Beq(_, _, _) => None,
            Instruction::Bne(_, _, _) => None,
            Instruction::Addiu(a, _, _) => Some(*a),
            Instruction::Andi(a, _, _) => Some(*a),
            Instruction::Ori(a, _, _) => Some(*a),
            Instruction::Lui(a, _) => Some(*a),
            Instruction::Ei => None,
            Instruction::Sq(_, _, _) => None,
            Instruction::Lh(a, _, _) => Some(*a),
            Instruction::Lw(a, _, _) => Some(*a),
            Instruction::Lbu(a, _, _) => Some(*a),
            Instruction::Lwr(a, _, _) => Some(*a),
            Instruction::Sb(_, _, _) => None,
            Instruction::Sh(_, _, _) => None,
            Instruction::Sw(_, _, _) => None,
            Instruction::Ld(a, _, _) => Some(*a),
            Instruction::Sd(_, _, _) => None,
        })
        .into_iter()
        .filter_map(|x| x.non_zero())
    }

    fn uses(&self) -> impl Iterator<Item = Register> {
        (match self {
            Instruction::Unknown => [None, None],
            Instruction::Sll(_, b, _) => [Some(*b), None],
            Instruction::Srl(_, b, _) => [Some(*b), None],
            Instruction::Sra(_, b, _) => [Some(*b), None],
            Instruction::Sllv(_, b, c) => [Some(*b), Some(*c)],
            Instruction::Srav(_, b, c) => [Some(*b), Some(*c)],
            Instruction::Jr(a) => [Some(*a), None],
            Instruction::Jalr(_, b) => [Some(*b), None],
            Instruction::Movz(_, b, c) => [Some(*b), Some(*c)],
            Instruction::Syscall => [None, None],
            Instruction::Sync => [None, None],
            Instruction::Mfhi(_) => [None, None],
            Instruction::Mthi(a) => [Some(*a), None],
            Instruction::Mflo(_) => [None, None],
            Instruction::Mult(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Add(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Addu(_, a, b) => [Some(*a), Some(*b)],
            Instruction::And(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Or(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Mtsa(a) => [Some(*a), None], // TODO: check
            Instruction::Slt(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Sltu(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Daddu(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Dsub(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Dsubu(_, a, b) => [Some(*a), Some(*b)],
            Instruction::Tge(a, b) => [Some(*a), Some(*b)],
            Instruction::Tgeu(a, b) => [Some(*a), Some(*b)],
            Instruction::Tlt(a, b) => [Some(*a), Some(*b)],
            Instruction::Tltu(a, b) => [Some(*a), Some(*b)],
            Instruction::Teq(a, b) => [Some(*a), Some(*b)],
            Instruction::Dsll(_, b, _) => [Some(*b), None],
            Instruction::Dsrl(_, b, _) => [Some(*b), None],
            Instruction::Dsra(_, b, _) => [Some(*b), None],
            Instruction::Dsll32(_, b, _) => [Some(*b), None],
            Instruction::Dsrl32(_, b, _) => [Some(*b), None],
            Instruction::Bgez(a, _) => [Some(*a), None],
            Instruction::J(_) => [None, None],
            Instruction::Jal(_) => [None, None],
            Instruction::Beq(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Bne(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Addiu(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Andi(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Ori(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Lui(_, _) => [None, None],
            Instruction::Ei => [None, None],
            Instruction::Sq(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Lh(_, b, _) => [Some(*b), None],
            Instruction::Lw(_, b, _) => [Some(*b), None],
            Instruction::Lbu(_, b, _) => [Some(*b), None],
            Instruction::Lwr(_, b, _) => [Some(*b), None],
            Instruction::Sb(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Sh(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Sw(a, b, _) => [Some(*a), Some(*b)],
            Instruction::Ld(_, b, _) => [Some(*b), None],
            Instruction::Sd(a, b, _) => [Some(*a), Some(*b)],
        })
        .into_iter()
        .take_while(|x| x.is_some())
        .filter_map(|x| x.and_then(|x| x.non_zero()))
    }

    fn depends_on(&self, register: Register) -> bool {
        self.uses().any(|x| x == register)
    }

    fn depends_on_instruction(&self, other: &Instruction) -> bool {
        other.definitions().any(|x| self.depends_on(x))
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Unknown => write!(f, "[unknown]"),
            Instruction::Sll(a, b, c) => write!(f, "sll {a}, {b}, {c}"),
            Instruction::Srl(a, b, c) => write!(f, "srl {a}, {b}, {c}"),
            Instruction::Sra(a, b, c) => write!(f, "sra {a}, {b}, {c}"),
            Instruction::Sllv(a, b, c) => write!(f, "sllv {a}, {b}, {c}"),
            Instruction::Srav(a, b, c) => write!(f, "srav {a}, {b}, {c}"),
            Instruction::Jr(a) => write!(f, "jr {a}"),
            Instruction::Jalr(a, b) => write!(f, "jalr {a}, {b}"),
            Instruction::Movz(a, b, c) => write!(f, "movz {a}, {b}, {c}"),
            Instruction::Syscall => write!(f, "syscall"),
            Instruction::Sync => write!(f, "sync"),
            Instruction::Mfhi(a) => write!(f, "mfhi {a}"),
            Instruction::Mthi(a) => write!(f, "mthi {a}"),
            Instruction::Mflo(a) => write!(f, "mflo {a}"),
            Instruction::Mult(a, b, c) => write!(f, "mult {a}, {b}, {c}"),
            Instruction::Add(a, b, c) => write!(f, "add {a}, {b}, {c}"),
            Instruction::Addu(a, b, c) => write!(f, "addu {a}, {b}, {c}"),
            Instruction::And(a, b, c) => write!(f, "and {a}, {b}, {c}"),
            Instruction::Or(a, b, c) => write!(f, "or {a}, {b}, {c}"),
            Instruction::Mtsa(a) => write!(f, "mtsa {a}"),
            Instruction::Slt(a, b, c) => write!(f, "slt {a}, {b}, {c}"),
            Instruction::Sltu(a, b, c) => write!(f, "sltu {a}, {b}, {c}"),
            Instruction::Daddu(a, b, c) => write!(f, "daddu {a}, {b}, {c}"),
            Instruction::Dsub(a, b, c) => write!(f, "dsub {a}, {b}, {c}"),
            Instruction::Dsubu(a, b, c) => write!(f, "dsubu {a}, {b}, {c}"),
            Instruction::Tge(a, b) => write!(f, "tge {a}, {b}"),
            Instruction::Tgeu(a, b) => write!(f, "tgeu {a}, {b}"),
            Instruction::Tlt(a, b) => write!(f, "tlt {a}, {b}"),
            Instruction::Tltu(a, b) => write!(f, "tltu {a}, {b}"),
            Instruction::Teq(a, b) => write!(f, "teq {a}, {b}"),
            Instruction::Dsll(a, b, c) => write!(f, "dsll {a}, {b}, {c}"),
            Instruction::Dsrl(a, b, c) => write!(f, "dsrl {a}, {b}, {c}"),
            Instruction::Dsra(a, b, c) => write!(f, "dsra {a}, {b}, {c}"),
            Instruction::Dsll32(a, b, c) => write!(f, "dsll32 {a}, {b}, {c}"),
            Instruction::Dsrl32(a, b, c) => write!(f, "dsrl32 {a}, {b}, {c}"),
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
            Instruction::Unknown => true,
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
        0b000000 => match data & 0b111111 {
            0b000000 => Instruction::Sll(rd, rt, shamt),
            0b000001 => Instruction::Unknown,
            0b000010 => Instruction::Srl(rd, rt, shamt),
            0b000011 => Instruction::Sra(rd, rt, shamt),
            0b000100 => Instruction::Sllv(rd, rt, rs),
            0b000101 => Instruction::Unknown,
            0b000111 => Instruction::Srav(rd, rt, rs),
            0b001000 => Instruction::Jr(rs),
            0b001001 => Instruction::Jalr(rd, rs),
            0b001010 => Instruction::Movz(rd, rs, rt),
            0b001100 => Instruction::Syscall,
            0b001111 => Instruction::Sync,
            0b010000 => Instruction::Mfhi(rd),
            0b010001 => Instruction::Mthi(rs),
            0b010010 => Instruction::Mflo(rd),
            0b010101 => Instruction::Unknown,
            0b011000 => Instruction::Mult(rs, rt, rd),
            0b100000 => Instruction::Add(rd, rs, rt),
            0b100001 => Instruction::Addu(rd, rs, rt),
            0b100100 => Instruction::And(rd, rs, rt),
            0b100101 => Instruction::Or(rd, rs, rt),
            0b101001 => Instruction::Mtsa(rs),
            0b101010 => Instruction::Slt(rd, rs, rt),
            0b101011 => Instruction::Sltu(rd, rs, rt),
            0b101101 => Instruction::Daddu(rd, rs, rt),
            0b101110 => Instruction::Dsub(rd, rs, rt),
            0b101111 => Instruction::Dsubu(rd, rs, rt),
            0b110000 => Instruction::Tge(rs, rt),
            0b110001 => Instruction::Tgeu(rs, rt),
            0b110010 => Instruction::Tlt(rs, rt),
            0b110011 => Instruction::Tltu(rs, rt),
            0b110100 => Instruction::Teq(rs, rt),
            0b110101 => Instruction::Unknown,
            0b110111 => Instruction::Unknown,
            0b111000 => Instruction::Dsll(rd, rs, shamt),
            0b111010 => Instruction::Dsrl(rd, rs, shamt),
            0b111011 => Instruction::Dsra(rd, rs, shamt),
            0b111100 => Instruction::Dsll32(rd, rs, shamt),
            0b111110 => Instruction::Dsrl32(rd, rs, shamt),
            _ => panic!("Special not implemented {:#034b}", data),
        },
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
