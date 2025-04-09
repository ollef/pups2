use std::fmt::Display;

use crate::bits::SignExtend;

use super::{
    fpu,
    register::{AnyRegister, Register},
};

#[derive(Debug)]
pub enum Instruction {
    Unknown,
    Sll(Register, Register, u8),
    Srl(Register, Register, u8),
    Sra(Register, Register, u8),
    Sllv(Register, Register, Register),
    Srlv(Register, Register, Register),
    Srav(Register, Register, Register),
    Jr(Register),
    Jalr(Register, Register),
    Movz(Register, Register, Register),
    Movn(Register, Register, Register),
    Syscall,
    Break,
    Sync,
    Mfhi(Register),
    Mthi(Register),
    Mflo(Register),
    Mtlo(Register),
    Dsllv(Register, Register, Register),
    Dsrav(Register, Register, Register),
    Dsrlv(Register, Register, Register),
    Mult(Register, Register, Register),
    Multu(Register, Register, Register),
    Div(Register, Register),
    Divu(Register, Register),
    Add(Register, Register, Register),
    Addu(Register, Register, Register),
    Sub(Register, Register, Register),
    Subu(Register, Register, Register),
    And(Register, Register, Register),
    Or(Register, Register, Register),
    Xor(Register, Register, Register),
    Nor(Register, Register, Register),
    Mfsa(Register),
    Mtsa(Register),
    Slt(Register, Register, Register),
    Sltu(Register, Register, Register),
    Dadd(Register, Register, Register),
    Daddu(Register, Register, Register),
    Dsub(Register, Register, Register),
    Dsubu(Register, Register, Register),
    Tge(Register, Register),
    Tgeu(Register, Register),
    Tlt(Register, Register),
    Tltu(Register, Register),
    Teq(Register, Register),
    Tne(Register, Register),
    Dsll(Register, Register, u8),
    Dsrl(Register, Register, u8),
    Dsra(Register, Register, u8),
    Dsll32(Register, Register, u8),
    Dsrl32(Register, Register, u8),
    Dsra32(Register, Register, u8),
    Bltz(Register, u16),
    Bgez(Register, u16),
    J(u32),
    Jal(u32),
    Beq(Register, Register, u16),
    Bne(Register, Register, u16),
    Blez(Register, u16),
    Addi(Register, Register, u16),
    Addiu(Register, Register, u16),
    Slti(Register, Register, u16),
    Sltiu(Register, Register, u16),
    Andi(Register, Register, u16),
    Ori(Register, Register, u16),
    Xori(Register, Register, u16),
    Lui(Register, u16),
    Mfc1(Register, fpu::Register),
    Mtc1(Register, fpu::Register),
    Muls(fpu::Register, fpu::Register, fpu::Register),
    Divs(fpu::Register, fpu::Register, fpu::Register),
    Movs(fpu::Register, fpu::Register),
    Cvtws(fpu::Register, fpu::Register),
    Cvtsw(fpu::Register, fpu::Register),
    Ei,
    Beql(Register, Register, u16),
    Sq(Register, Register, u16),
    Lb(Register, Register, u16),
    Lh(Register, Register, u16),
    Lw(Register, Register, u16),
    Lbu(Register, Register, u16),
    Lhu(Register, Register, u16),
    Lwr(Register, Register, u16),
    Sb(Register, Register, u16),
    Sh(Register, Register, u16),
    Sw(Register, Register, u16),
    Lwc1(fpu::Register, Register, u16),
    Ld(Register, Register, u16),
    Swc1(fpu::Register, Register, u16),
    Sd(Register, Register, u16),
}

impl Instruction {
    pub fn definitions(&self) -> impl Iterator<Item = AnyRegister> {
        let gpr = |x| Some(AnyRegister::Core(x));
        let fpr = |x| Some(AnyRegister::Fpu(x));
        (match self {
            Instruction::Unknown => [None, None, None],
            Instruction::Sll(a, _, _) => [gpr(*a), None, None],
            Instruction::Srl(a, _, _) => [gpr(*a), None, None],
            Instruction::Sra(a, _, _) => [gpr(*a), None, None],
            Instruction::Sllv(a, _, _) => [gpr(*a), None, None],
            Instruction::Srlv(a, _, _) => [gpr(*a), None, None],
            Instruction::Srav(a, _, _) => [gpr(*a), None, None],
            Instruction::Jr(_) => [None, None, None],
            Instruction::Jalr(a, _) => [gpr(*a), None, None],
            Instruction::Movz(a, _, _) => [gpr(*a), None, None],
            Instruction::Movn(a, _, _) => [gpr(*a), None, None],
            Instruction::Syscall => [None, None, None],
            Instruction::Break => [None, None, None],
            Instruction::Sync => [None, None, None],
            Instruction::Mfhi(a) => [gpr(*a), None, None],
            Instruction::Mthi(_) => [None, None, None],
            Instruction::Mflo(a) => [gpr(*a), None, None],
            Instruction::Mtlo(_) => [None, None, None],
            Instruction::Dsllv(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsrav(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsrlv(a, _, _) => [gpr(*a), None, None],
            Instruction::Mult(a, _, _) => [gpr(*a), gpr(Register::Lo), gpr(Register::Hi)],
            Instruction::Multu(a, _, _) => [gpr(*a), gpr(Register::Lo), gpr(Register::Hi)],
            Instruction::Div(_, _) => [gpr(Register::Lo), gpr(Register::Hi), None],
            Instruction::Divu(_, _) => [gpr(Register::Lo), gpr(Register::Hi), None],
            Instruction::Add(a, _, _) => [gpr(*a), None, None],
            Instruction::Addu(a, _, _) => [gpr(*a), None, None],
            Instruction::Sub(a, _, _) => [gpr(*a), None, None],
            Instruction::Subu(a, _, _) => [gpr(*a), None, None],
            Instruction::And(a, _, _) => [gpr(*a), None, None],
            Instruction::Or(a, _, _) => [gpr(*a), None, None],
            Instruction::Xor(a, _, _) => [gpr(*a), None, None],
            Instruction::Nor(a, _, _) => [gpr(*a), None, None],
            Instruction::Mfsa(a) => [gpr(*a), None, None],
            Instruction::Mtsa(_) => [None, None, None],
            Instruction::Slt(a, _, _) => [gpr(*a), None, None],
            Instruction::Sltu(a, _, _) => [gpr(*a), None, None],
            Instruction::Dadd(a, _, _) => [gpr(*a), None, None],
            Instruction::Daddu(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsub(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsubu(a, _, _) => [gpr(*a), None, None],
            Instruction::Tge(_, _) => [None, None, None],
            Instruction::Tgeu(_, _) => [None, None, None],
            Instruction::Tlt(_, _) => [None, None, None],
            Instruction::Tltu(_, _) => [None, None, None],
            Instruction::Teq(_, _) => [None, None, None],
            Instruction::Tne(_, _) => [None, None, None],
            Instruction::Dsll(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsrl(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsra(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsll32(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsrl32(a, _, _) => [gpr(*a), None, None],
            Instruction::Dsra32(a, _, _) => [gpr(*a), None, None],
            Instruction::Bltz(_, _) => [None, None, None],
            Instruction::Bgez(_, _) => [None, None, None],
            Instruction::J(_) => [None, None, None],
            Instruction::Jal(_) => [gpr(Register::Ra), None, None],
            Instruction::Beq(_, _, _) => [None, None, None],
            Instruction::Bne(_, _, _) => [None, None, None],
            Instruction::Blez(_, _) => [None, None, None],
            Instruction::Addi(a, _, _) => [gpr(*a), None, None],
            Instruction::Addiu(a, _, _) => [gpr(*a), None, None],
            Instruction::Slti(a, _, _) => [gpr(*a), None, None],
            Instruction::Sltiu(a, _, _) => [gpr(*a), None, None],
            Instruction::Andi(a, _, _) => [gpr(*a), None, None],
            Instruction::Ori(a, _, _) => [gpr(*a), None, None],
            Instruction::Xori(a, _, _) => [gpr(*a), None, None],
            Instruction::Lui(a, _) => [gpr(*a), None, None],
            Instruction::Mfc1(a, _) => [gpr(*a), None, None],
            Instruction::Mtc1(_, b) => [fpr(*b), None, None],
            Instruction::Muls(a, _, _) => [fpr(*a), None, None],
            Instruction::Divs(a, _, _) => [fpr(*a), None, None],
            Instruction::Movs(a, _) => [fpr(*a), None, None],
            Instruction::Cvtws(a, _) => [fpr(*a), None, None],
            Instruction::Cvtsw(a, _) => [fpr(*a), None, None],
            Instruction::Ei => [None, None, None],
            Instruction::Beql(_, _, _) => [None, None, None],
            Instruction::Sq(_, _, _) => [None, None, None],
            Instruction::Lb(a, _, _) => [gpr(*a), None, None],
            Instruction::Lh(a, _, _) => [gpr(*a), None, None],
            Instruction::Lw(a, _, _) => [gpr(*a), None, None],
            Instruction::Lbu(a, _, _) => [gpr(*a), None, None],
            Instruction::Lhu(a, _, _) => [gpr(*a), None, None],
            Instruction::Lwr(a, _, _) => [gpr(*a), None, None],
            Instruction::Sb(_, _, _) => [None, None, None],
            Instruction::Sh(_, _, _) => [None, None, None],
            Instruction::Sw(_, _, _) => [None, None, None],
            Instruction::Lwc1(a, _, _) => [fpr(*a), None, None],
            Instruction::Ld(a, _, _) => [gpr(*a), None, None],
            Instruction::Swc1(_, _, _) => [None, None, None],
            Instruction::Sd(_, _, _) => [None, None, None],
        })
        .into_iter()
        .take_while(|x| x.is_some())
        .filter_map(|x| x.and_then(|x| x.non_zero()))
    }

    pub fn uses(&self) -> impl Iterator<Item = AnyRegister> {
        let gpr = |x| Some(AnyRegister::Core(x));
        let fpr = |x| Some(AnyRegister::Fpu(x));
        (match self {
            Instruction::Unknown => [None, None],
            Instruction::Sll(_, b, _) => [gpr(*b), None],
            Instruction::Srl(_, b, _) => [gpr(*b), None],
            Instruction::Sra(_, b, _) => [gpr(*b), None],
            Instruction::Sllv(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Srlv(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Srav(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Jr(a) => [gpr(*a), None],
            Instruction::Jalr(_, b) => [gpr(*b), None],
            Instruction::Movz(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Movn(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Syscall => [None, None],
            Instruction::Break => [None, None],
            Instruction::Sync => [None, None],
            Instruction::Mfhi(_) => [None, None],
            Instruction::Mthi(a) => [gpr(*a), None],
            Instruction::Mflo(_) => [None, None],
            Instruction::Mtlo(a) => [gpr(*a), None],
            Instruction::Dsllv(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Dsrav(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Dsrlv(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Mult(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Multu(_, b, c) => [gpr(*b), gpr(*c)],
            Instruction::Div(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Divu(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Add(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Addu(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Sub(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Subu(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::And(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Or(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Xor(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Nor(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Mfsa(_) => [None, None],
            Instruction::Mtsa(a) => [gpr(*a), None],
            Instruction::Slt(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Sltu(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Dadd(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Daddu(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Dsub(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Dsubu(_, a, b) => [gpr(*a), gpr(*b)],
            Instruction::Tge(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Tgeu(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Tlt(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Tltu(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Teq(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Tne(a, b) => [gpr(*a), gpr(*b)],
            Instruction::Dsll(_, b, _) => [gpr(*b), None],
            Instruction::Dsrl(_, b, _) => [gpr(*b), None],
            Instruction::Dsra(_, b, _) => [gpr(*b), None],
            Instruction::Dsll32(_, b, _) => [gpr(*b), None],
            Instruction::Dsrl32(_, b, _) => [gpr(*b), None],
            Instruction::Dsra32(_, b, _) => [gpr(*b), None],
            Instruction::Bltz(a, _) => [gpr(*a), None],
            Instruction::Bgez(a, _) => [gpr(*a), None],
            Instruction::J(_) => [None, None],
            Instruction::Jal(_) => [None, None],
            Instruction::Beq(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Bne(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Blez(a, _) => [gpr(*a), None],
            Instruction::Addi(_, b, _) => [gpr(*b), None],
            Instruction::Addiu(_, b, _) => [gpr(*b), None],
            Instruction::Slti(_, b, _) => [gpr(*b), None],
            Instruction::Sltiu(_, b, _) => [gpr(*b), None],
            Instruction::Andi(_, b, _) => [gpr(*b), None],
            Instruction::Ori(_, b, _) => [gpr(*b), None],
            Instruction::Xori(_, b, _) => [gpr(*b), None],
            Instruction::Lui(_, _) => [None, None],
            Instruction::Mfc1(_, b) => [fpr(*b), None],
            Instruction::Mtc1(a, _) => [gpr(*a), None],
            Instruction::Muls(_, b, c) => [fpr(*b), fpr(*c)],
            Instruction::Divs(_, b, c) => [fpr(*b), fpr(*c)],
            Instruction::Movs(_, b) => [fpr(*b), None],
            Instruction::Cvtws(_, b) => [fpr(*b), None],
            Instruction::Cvtsw(_, b) => [fpr(*b), None],
            Instruction::Ei => [None, None],
            Instruction::Beql(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Sq(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Lb(_, b, _) => [gpr(*b), None],
            Instruction::Lh(_, b, _) => [gpr(*b), None],
            Instruction::Lw(_, b, _) => [gpr(*b), None],
            Instruction::Lbu(_, b, _) => [gpr(*b), None],
            Instruction::Lhu(_, b, _) => [gpr(*b), None],
            Instruction::Lwr(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Sb(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Sh(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Sw(a, b, _) => [gpr(*a), gpr(*b)],
            Instruction::Lwc1(_, b, _) => [gpr(*b), None],
            Instruction::Ld(_, b, _) => [gpr(*b), None],
            Instruction::Swc1(a, b, _) => [fpr(*a), gpr(*b)],
            Instruction::Sd(a, b, _) => [gpr(*a), gpr(*b)],
        })
        .into_iter()
        .take_while(|x| x.is_some())
        .filter_map(|x| x.and_then(|x| x.non_zero()))
    }

    fn depends_on(&self, register: AnyRegister) -> bool {
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
            Instruction::Srlv(a, b, c) => write!(f, "srlv {a}, {b}, {c}"),
            Instruction::Srav(a, b, c) => write!(f, "srav {a}, {b}, {c}"),
            Instruction::Jr(a) => write!(f, "jr {a}"),
            Instruction::Jalr(a, b) => write!(f, "jalr {a}, {b}"),
            Instruction::Movz(a, b, c) => write!(f, "movz {a}, {b}, {c}"),
            Instruction::Movn(a, b, c) => write!(f, "movn {a}, {b}, {c}"),
            Instruction::Syscall => write!(f, "syscall"),
            Instruction::Break => write!(f, "break"),
            Instruction::Sync => write!(f, "sync"),
            Instruction::Mfhi(a) => write!(f, "mfhi {a}"),
            Instruction::Mthi(a) => write!(f, "mthi {a}"),
            Instruction::Mflo(a) => write!(f, "mflo {a}"),
            Instruction::Mtlo(a) => write!(f, "mtlo {a}"),
            Instruction::Dsllv(a, b, c) => write!(f, "dsllv {a}, {b}, {c}"),
            Instruction::Dsrav(a, b, c) => write!(f, "dsrav {a}, {b}, {c}"),
            Instruction::Dsrlv(a, b, c) => write!(f, "dsrlv {a}, {b}, {c}"),
            Instruction::Mult(a, b, c) => write!(f, "mult {a}, {b}, {c}"),
            Instruction::Multu(a, b, c) => write!(f, "multu {a}, {b}, {c}"),
            Instruction::Div(a, b) => write!(f, "div {a}, {b}"),
            Instruction::Divu(a, b) => write!(f, "divu {a}, {b}"),
            Instruction::Add(a, b, c) => write!(f, "add {a}, {b}, {c}"),
            Instruction::Addu(a, b, c) => write!(f, "addu {a}, {b}, {c}"),
            Instruction::Sub(a, b, c) => write!(f, "sub {a}, {b}, {c}"),
            Instruction::Subu(a, b, c) => write!(f, "subu {a}, {b}, {c}"),
            Instruction::And(a, b, c) => write!(f, "and {a}, {b}, {c}"),
            Instruction::Or(a, b, c) => write!(f, "or {a}, {b}, {c}"),
            Instruction::Xor(a, b, c) => write!(f, "xor {a}, {b}, {c}"),
            Instruction::Nor(a, b, c) => write!(f, "nor {a}, {b}, {c}"),
            Instruction::Mfsa(a) => write!(f, "mfsa {a}"),
            Instruction::Mtsa(a) => write!(f, "mtsa {a}"),
            Instruction::Slt(a, b, c) => write!(f, "slt {a}, {b}, {c}"),
            Instruction::Sltu(a, b, c) => write!(f, "sltu {a}, {b}, {c}"),
            Instruction::Dadd(a, b, c) => write!(f, "dadd {a}, {b}, {c}"),
            Instruction::Daddu(a, b, c) => write!(f, "daddu {a}, {b}, {c}"),
            Instruction::Dsub(a, b, c) => write!(f, "dsub {a}, {b}, {c}"),
            Instruction::Dsubu(a, b, c) => write!(f, "dsubu {a}, {b}, {c}"),
            Instruction::Tge(a, b) => write!(f, "tge {a}, {b}"),
            Instruction::Tgeu(a, b) => write!(f, "tgeu {a}, {b}"),
            Instruction::Tlt(a, b) => write!(f, "tlt {a}, {b}"),
            Instruction::Tltu(a, b) => write!(f, "tltu {a}, {b}"),
            Instruction::Teq(a, b) => write!(f, "teq {a}, {b}"),
            Instruction::Tne(a, b) => write!(f, "tne {a}, {b}"),
            Instruction::Dsll(a, b, c) => write!(f, "dsll {a}, {b}, {c}"),
            Instruction::Dsrl(a, b, c) => write!(f, "dsrl {a}, {b}, {c}"),
            Instruction::Dsra(a, b, c) => write!(f, "dsra {a}, {b}, {c}"),
            Instruction::Dsll32(a, b, c) => write!(f, "dsll32 {a}, {b}, {c}"),
            Instruction::Dsrl32(a, b, c) => write!(f, "dsrl32 {a}, {b}, {c}"),
            Instruction::Dsra32(a, b, c) => write!(f, "dsra32 {a}, {b}, {c}"),
            Instruction::Bltz(a, b) => write!(f, "bltz {a}, {b:#x}"),
            Instruction::Bgez(a, b) => write!(f, "bgez {a}, {b:#x}"),
            Instruction::J(a) => write!(f, "j {a:#x}"),
            Instruction::Jal(a) => write!(f, "jal {a:#x}"),
            Instruction::Beq(a, b, c) => write!(f, "beq {a}, {b}, {c:#x}"),
            Instruction::Bne(a, b, c) => write!(f, "bne {a}, {b}, {c:#x}"),
            Instruction::Blez(a, b) => write!(f, "blez {a}, {b:#x}"),
            Instruction::Addi(a, b, c) => write!(f, "addi {a}, {b}, {c:#x}"),
            Instruction::Addiu(a, b, c) => write!(f, "addiu {a}, {b}, {c:#x}"),
            Instruction::Slti(a, b, c) => write!(f, "slti {a}, {b}, {c:#x}"),
            Instruction::Sltiu(a, b, c) => write!(f, "sltiu {a}, {b}, {c:#x}"),
            Instruction::Andi(a, b, c) => write!(f, "andi {a}, {b}, {c:#x}"),
            Instruction::Ori(a, b, c) => write!(f, "ori {a}, {b}, {c:#x}"),
            Instruction::Xori(a, b, c) => write!(f, "xori {a}, {b}, {c:#x}"),
            Instruction::Lui(a, b) => write!(f, "lui {a}, {b}"),
            Instruction::Mfc1(a, b) => write!(f, "mfc1 {a}, {b}"),
            Instruction::Mtc1(a, b) => write!(f, "mtc1 {a}, {b}"),
            Instruction::Muls(a, b, c) => write!(f, "mul.s {a}, {b}, {c}"),
            Instruction::Divs(a, b, c) => write!(f, "div.s {a}, {b}, {c}"),
            Instruction::Movs(a, b) => write!(f, "mov.s {a}, {b}"),
            Instruction::Cvtws(a, b) => write!(f, "cvt.w.s {a}, {b}"),
            Instruction::Cvtsw(a, b) => write!(f, "cvt.s.w {a}, {b}"),
            Instruction::Ei => write!(f, "ei"),
            Instruction::Beql(a, b, c) => write!(f, "beql {a}, {c:#x}({b})"),
            Instruction::Sq(a, b, c) => write!(f, "sq {a}, {c:#x}({b})"),
            Instruction::Lb(a, b, c) => write!(f, "lb {a}, {c:#x}({b})"),
            Instruction::Lh(a, b, c) => write!(f, "lh {a}, {c:#x}({b})"),
            Instruction::Lw(a, b, c) => write!(f, "lw {a}, {c:#x}({b})"),
            Instruction::Lbu(a, b, c) => write!(f, "lbu {a}, {c:#x}({b})"),
            Instruction::Lhu(a, b, c) => write!(f, "lhu {a}, {c:#x}({b})"),
            Instruction::Lwr(a, b, c) => write!(f, "lwr {a}, {c:#x}({b})"),
            Instruction::Sb(a, b, c) => write!(f, "sb {a}, {c:#x}({b})"),
            Instruction::Sh(a, b, c) => write!(f, "sh {a}, {c:#x}({b})"),
            Instruction::Sw(a, b, c) => write!(f, "sw {a}, {c:#x}({b})"),
            Instruction::Lwc1(a, b, c) => write!(f, "lwc1 {a}, {c:#x}({b})"),
            Instruction::Ld(a, b, c) => write!(f, "ld {a}, {c:#x}({b})"),
            Instruction::Swc1(a, b, c) => write!(f, "swc1 {a}, {c:#x}({b})"),
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

    pub fn branch_target(&self, address: u32) -> Option<u32> {
        match self {
            Instruction::Bltz(_, offset)
            | Instruction::Bgez(_, offset)
            | Instruction::Beq(_, _, offset)
            | Instruction::Bne(_, _, offset)
            | Instruction::Blez(_, offset)
            | Instruction::Beql(_, _, offset) => Some({
                let offset: u32 = offset.sign_extend();
                address.wrapping_add(4).wrapping_add(offset << 2)
            }),
            Instruction::J(target) | Instruction::Jal(target) => {
                Some(((address + 4) & 0xF000_0000).wrapping_add(target << 2))
            }
            _ => None,
        }
    }
}
