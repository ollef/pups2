// Generated file. Do not edit!
use super::control;
use super::fpu;
use super::instruction::Occurrence;
use super::register::Register;
use crate::bits::Bits;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Instruction {
    Sll(Register, Register, u8),
    Unknown,
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
    Dsrlv(Register, Register, Register),
    Dsrav(Register, Register, Register),
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
    Mfc0(Register, control::Register),
    Mtc0(control::Register, Register),
    Tlbr,
    Tlbwi,
    Tlbwr,
    Tlbp,
    Ei,
    Mfc1(Register, fpu::Register),
    Mtc1(fpu::Register, Register),
    Muls(fpu::Register, fpu::Register, fpu::Register),
    Divs(fpu::Register, fpu::Register, fpu::Register),
    Movs(fpu::Register, fpu::Register),
    Cvtws(fpu::Register, fpu::Register),
    Cvtsw(fpu::Register, fpu::Register),
    Beql(Register, Register, u16),
    Bnel(Register, Register, u16),
    Mfhi1(Register),
    Mthi1(Register),
    Mflo1(Register),
    Mtlo1(Register),
    Div1(Register, Register),
    Divu1(Register, Register),
    Sq(Register, u16, Register),
    Lb(Register, u16, Register),
    Lh(Register, u16, Register),
    Lw(Register, u16, Register),
    Lbu(Register, u16, Register),
    Lhu(Register, u16, Register),
    Lwr(Register, u16, Register),
    Sb(Register, u16, Register),
    Sh(Register, u16, Register),
    Sw(Register, u16, Register),
    Lwc1(fpu::Register, u16, Register),
    Ld(Register, u16, Register),
    Swc1(fpu::Register, u16, Register),
    Sd(Register, u16, Register),
}

impl Instruction {
    pub fn decode(data: u32) -> Self {
        let rs = || Register::from(data.bits(21..26));
        let rt = || Register::from(data.bits(16..21));
        let rd = || Register::from(data.bits(11..16));
        let ft = || fpu::Register::from(data.bits(16..21));
        let fs = || fpu::Register::from(data.bits(11..16));
        let fd = || fpu::Register::from(data.bits(6..11));
        let cd = || control::Register::from(data.bits(11..16));
        let sa = || data.bits(6..11) as u8;
        let imm16 = || data.bits(0..16) as u16;
        let imm26 = || data.bits(0..26);
        match data.bits(26..32) {
            0b000000 => match data.bits(0..6) {
                0b000000 => match data.bits(21..26) {
                    0b00000 => Instruction::Sll(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000001 => Instruction::Unknown,
                0b000010 => match data.bits(21..26) {
                    0b00000 => Instruction::Srl(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000011 => match data.bits(21..26) {
                    0b00000 => Instruction::Sra(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000100 => match data.bits(6..11) {
                    0b00000 => Instruction::Sllv(rd(), rt(), rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000101 => Instruction::Unknown,
                0b000110 => match data.bits(6..11) {
                    0b00000 => Instruction::Srlv(rd(), rt(), rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000111 => match data.bits(6..11) {
                    0b00000 => Instruction::Srav(rd(), rt(), rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b001000 => match data.bits(6..21) {
                    0b000000000000000 => Instruction::Jr(rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b001001 => match data.bits(6..11) {
                    0b00000 => match data.bits(16..21) {
                        0b00000 => Instruction::Jalr(rd(), rs()),
                        _ => panic!("Unhandled instruction: {:#034b}", data),
                    }
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b001010 => match data.bits(6..11) {
                    0b00000 => Instruction::Movz(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b001011 => match data.bits(6..11) {
                    0b00000 => Instruction::Movn(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b001100 => Instruction::Syscall,
                0b001101 => Instruction::Break,
                0b001110 => Instruction::Unknown,
                0b001111 => match data.bits(11..26) {
                    0b000000000000000 => Instruction::Sync,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010000 => match data.bits(6..11) {
                    0b00000 => match data.bits(16..26) {
                        0b0000000000 => Instruction::Mfhi(rd()),
                        _ => panic!("Unhandled instruction: {:#034b}", data),
                    }
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010001 => match data.bits(6..21) {
                    0b000000000000000 => Instruction::Mthi(rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010010 => match data.bits(6..11) {
                    0b00000 => match data.bits(16..26) {
                        0b0000000000 => Instruction::Mflo(rd()),
                        _ => panic!("Unhandled instruction: {:#034b}", data),
                    }
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010011 => match data.bits(6..21) {
                    0b000000000000000 => Instruction::Mtlo(rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010100 => match data.bits(6..11) {
                    0b00000 => Instruction::Dsllv(rd(), rt(), rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010101 => Instruction::Unknown,
                0b010110 => match data.bits(6..11) {
                    0b00000 => Instruction::Dsrlv(rd(), rt(), rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b010111 => match data.bits(6..11) {
                    0b00000 => Instruction::Dsrav(rd(), rt(), rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b011000 => match data.bits(6..11) {
                    0b00000 => Instruction::Mult(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b011001 => match data.bits(6..11) {
                    0b00000 => Instruction::Multu(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b011010 => match data.bits(6..16) {
                    0b0000000000 => Instruction::Div(rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b011011 => match data.bits(6..16) {
                    0b0000000000 => Instruction::Divu(rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b011100 => Instruction::Unknown,
                0b011101 => Instruction::Unknown,
                0b011110 => Instruction::Unknown,
                0b011111 => Instruction::Unknown,
                0b100000 => match data.bits(6..11) {
                    0b00000 => Instruction::Add(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100001 => match data.bits(6..11) {
                    0b00000 => Instruction::Addu(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100010 => match data.bits(6..11) {
                    0b00000 => Instruction::Sub(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100011 => match data.bits(6..11) {
                    0b00000 => Instruction::Subu(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100100 => match data.bits(6..11) {
                    0b00000 => Instruction::And(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100101 => match data.bits(6..11) {
                    0b00000 => Instruction::Or(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100110 => match data.bits(6..11) {
                    0b00000 => Instruction::Xor(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100111 => match data.bits(6..11) {
                    0b00000 => Instruction::Nor(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101000 => match data.bits(6..11) {
                    0b00000 => match data.bits(16..26) {
                        0b0000000000 => Instruction::Mfsa(rd()),
                        _ => panic!("Unhandled instruction: {:#034b}", data),
                    }
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101001 => match data.bits(6..21) {
                    0b000000000000000 => Instruction::Mtsa(rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101010 => match data.bits(6..11) {
                    0b00000 => Instruction::Slt(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101011 => match data.bits(6..11) {
                    0b00000 => Instruction::Sltu(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101100 => match data.bits(6..11) {
                    0b00000 => Instruction::Dadd(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101101 => match data.bits(6..11) {
                    0b00000 => Instruction::Daddu(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101110 => match data.bits(6..11) {
                    0b00000 => Instruction::Dsub(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b101111 => match data.bits(6..11) {
                    0b00000 => Instruction::Dsubu(rd(), rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b110000 => Instruction::Tge(rs(), rt()),
                0b110001 => Instruction::Tgeu(rs(), rt()),
                0b110010 => Instruction::Tlt(rs(), rt()),
                0b110011 => Instruction::Tltu(rs(), rt()),
                0b110100 => Instruction::Teq(rs(), rt()),
                0b110101 => Instruction::Unknown,
                0b110110 => Instruction::Tne(rs(), rt()),
                0b110111 => Instruction::Unknown,
                0b111000 => match data.bits(21..26) {
                    0b00000 => Instruction::Dsll(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b111001 => Instruction::Unknown,
                0b111010 => match data.bits(21..26) {
                    0b00000 => Instruction::Dsrl(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b111011 => match data.bits(21..26) {
                    0b00000 => Instruction::Dsra(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b111100 => match data.bits(21..26) {
                    0b00000 => Instruction::Dsll32(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b111101 => Instruction::Unknown,
                0b111110 => match data.bits(21..26) {
                    0b00000 => Instruction::Dsrl32(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b111111 => match data.bits(21..26) {
                    0b00000 => Instruction::Dsra32(rd(), rt(), sa()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                _ => unreachable!(),
            }
            0b000001 => match data.bits(16..21) {
                0b00000 => Instruction::Bltz(rs(), imm16()),
                0b00001 => Instruction::Bgez(rs(), imm16()),
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b000010 => Instruction::J(imm26()),
            0b000011 => Instruction::Jal(imm26()),
            0b000100 => Instruction::Beq(rs(), rt(), imm16()),
            0b000101 => Instruction::Bne(rs(), rt(), imm16()),
            0b000110 => match data.bits(16..21) {
                0b00000 => Instruction::Blez(rs(), imm16()),
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b001000 => Instruction::Addi(rt(), rs(), imm16()),
            0b001001 => Instruction::Addiu(rt(), rs(), imm16()),
            0b001010 => Instruction::Slti(rt(), rs(), imm16()),
            0b001011 => Instruction::Sltiu(rt(), rs(), imm16()),
            0b001100 => Instruction::Andi(rt(), rs(), imm16()),
            0b001101 => Instruction::Ori(rt(), rs(), imm16()),
            0b001110 => Instruction::Xori(rt(), rs(), imm16()),
            0b001111 => match data.bits(21..26) {
                0b00000 => Instruction::Lui(rt(), imm16()),
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b010000 => match data.bits(0..11) {
                0b00000000000 => match data.bits(21..26) {
                    0b00000 => Instruction::Mfc0(rt(), cd()),
                    0b00100 => Instruction::Mtc0(cd(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000000001 => match data.bits(11..26) {
                    0b100000000000000 => Instruction::Tlbr,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000000010 => match data.bits(11..26) {
                    0b100000000000000 => Instruction::Tlbwi,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000000110 => match data.bits(11..26) {
                    0b100000000000000 => Instruction::Tlbwr,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000001000 => match data.bits(11..26) {
                    0b100000000000000 => Instruction::Tlbp,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000111000 => match data.bits(11..26) {
                    0b100000000000000 => Instruction::Ei,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b010001 => match data.bits(0..6) {
                0b000000 => match data.bits(6..11) {
                    0b00000 => match data.bits(21..26) {
                        0b00000 => Instruction::Mfc1(rt(), fs()),
                        0b00100 => Instruction::Mtc1(fs(), rt()),
                        _ => panic!("Unhandled instruction: {:#034b}", data),
                    }
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000010 => match data.bits(21..26) {
                    0b10000 => Instruction::Muls(fd(), fs(), ft()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000011 => match data.bits(21..26) {
                    0b10000 => Instruction::Divs(fd(), fs(), ft()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000110 => match data.bits(16..26) {
                    0b1000000000 => Instruction::Movs(fd(), fs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100000 => match data.bits(16..26) {
                    0b1010000000 => Instruction::Cvtsw(fd(), fs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100100 => match data.bits(16..26) {
                    0b1000000000 => Instruction::Cvtws(fd(), fs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b010100 => Instruction::Beql(rs(), rt(), imm16()),
            0b010101 => Instruction::Bnel(rs(), rt(), imm16()),
            0b011100 => match data.bits(0..11) {
                0b00000010000 => match data.bits(16..26) {
                    0b0000000000 => Instruction::Mfhi1(rd()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000010001 => match data.bits(11..21) {
                    0b0000000000 => Instruction::Mthi1(rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000010010 => match data.bits(16..26) {
                    0b0000000000 => Instruction::Mflo1(rd()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000010011 => match data.bits(11..21) {
                    0b0000000000 => Instruction::Mtlo1(rs()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000011010 => match data.bits(11..16) {
                    0b00000 => Instruction::Div1(rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000011011 => match data.bits(11..16) {
                    0b00000 => Instruction::Divu1(rs(), rt()),
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b011111 => Instruction::Sq(rt(), imm16(), rs()),
            0b100000 => Instruction::Lb(rt(), imm16(), rs()),
            0b100001 => Instruction::Lh(rt(), imm16(), rs()),
            0b100011 => Instruction::Lw(rt(), imm16(), rs()),
            0b100100 => Instruction::Lbu(rt(), imm16(), rs()),
            0b100101 => Instruction::Lhu(rt(), imm16(), rs()),
            0b100110 => Instruction::Lwr(rt(), imm16(), rs()),
            0b101000 => Instruction::Sb(rt(), imm16(), rs()),
            0b101001 => Instruction::Sh(rt(), imm16(), rs()),
            0b101011 => Instruction::Sw(rt(), imm16(), rs()),
            0b110001 => Instruction::Lwc1(ft(), imm16(), rs()),
            0b110111 => Instruction::Ld(rt(), imm16(), rs()),
            0b111001 => Instruction::Swc1(ft(), imm16(), rs()),
            0b111111 => Instruction::Sd(rt(), imm16(), rs()),
            _ => panic!("Unhandled instruction: {:#034b}", data),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Instruction::Sll(rd, rt, sa) => write!(f, "{rd} = sll {rt}, {sa}"),
            Instruction::Unknown => write!(f, "unknown"),
            Instruction::Srl(rd, rt, sa) => write!(f, "{rd} = srl {rt}, {sa}"),
            Instruction::Sra(rd, rt, sa) => write!(f, "{rd} = sra {rt}, {sa}"),
            Instruction::Sllv(rd, rt, rs) => write!(f, "{rd} = sllv {rt}, {rs}"),
            Instruction::Srlv(rd, rt, rs) => write!(f, "{rd} = srlv {rt}, {rs}"),
            Instruction::Srav(rd, rt, rs) => write!(f, "{rd} = srav {rt}, {rs}"),
            Instruction::Jr(rs) => write!(f, "jr {rs}"),
            Instruction::Jalr(rd, rs) => write!(f, "jalr {rd}, {rs}"),
            Instruction::Movz(rd, rs, rt) => write!(f, "{rd} = movz {rs}, {rt}"),
            Instruction::Movn(rd, rs, rt) => write!(f, "{rd} = movn {rs}, {rt}"),
            Instruction::Syscall => write!(f, "syscall"),
            Instruction::Break => write!(f, "break"),
            Instruction::Sync => write!(f, "sync"),
            Instruction::Mfhi(rd) => write!(f, "{rd} = mfhi"),
            Instruction::Mthi(rs) => write!(f, "mthi {rs}"),
            Instruction::Mflo(rd) => write!(f, "{rd} = mflo"),
            Instruction::Mtlo(rs) => write!(f, "mtlo {rs}"),
            Instruction::Dsllv(rd, rt, rs) => write!(f, "{rd} = dsllv {rt}, {rs}"),
            Instruction::Dsrlv(rd, rt, rs) => write!(f, "{rd} = dsrlv {rt}, {rs}"),
            Instruction::Dsrav(rd, rt, rs) => write!(f, "{rd} = dsrav {rt}, {rs}"),
            Instruction::Mult(rd, rs, rt) => write!(f, "{rd} = mult {rs}, {rt}"),
            Instruction::Multu(rd, rs, rt) => write!(f, "{rd} = multu {rs}, {rt}"),
            Instruction::Div(rs, rt) => write!(f, "div {rs}, {rt}"),
            Instruction::Divu(rs, rt) => write!(f, "divu {rs}, {rt}"),
            Instruction::Add(rd, rs, rt) => write!(f, "{rd} = add {rs}, {rt}"),
            Instruction::Addu(rd, rs, rt) => write!(f, "{rd} = addu {rs}, {rt}"),
            Instruction::Sub(rd, rs, rt) => write!(f, "{rd} = sub {rs}, {rt}"),
            Instruction::Subu(rd, rs, rt) => write!(f, "{rd} = subu {rs}, {rt}"),
            Instruction::And(rd, rs, rt) => write!(f, "{rd} = and {rs}, {rt}"),
            Instruction::Or(rd, rs, rt) => write!(f, "{rd} = or {rs}, {rt}"),
            Instruction::Xor(rd, rs, rt) => write!(f, "{rd} = xor {rs}, {rt}"),
            Instruction::Nor(rd, rs, rt) => write!(f, "{rd} = nor {rs}, {rt}"),
            Instruction::Mfsa(rd) => write!(f, "{rd} = mfsa"),
            Instruction::Mtsa(rs) => write!(f, "mtsa {rs}"),
            Instruction::Slt(rd, rs, rt) => write!(f, "{rd} = slt {rs}, {rt}"),
            Instruction::Sltu(rd, rs, rt) => write!(f, "{rd} = sltu {rs}, {rt}"),
            Instruction::Dadd(rd, rs, rt) => write!(f, "{rd} = dadd {rs}, {rt}"),
            Instruction::Daddu(rd, rs, rt) => write!(f, "{rd} = daddu {rs}, {rt}"),
            Instruction::Dsub(rd, rs, rt) => write!(f, "{rd} = dsub {rs}, {rt}"),
            Instruction::Dsubu(rd, rs, rt) => write!(f, "{rd} = dsubu {rs}, {rt}"),
            Instruction::Tge(rs, rt) => write!(f, "tge {rs}, {rt}"),
            Instruction::Tgeu(rs, rt) => write!(f, "tgeu {rs}, {rt}"),
            Instruction::Tlt(rs, rt) => write!(f, "tlt {rs}, {rt}"),
            Instruction::Tltu(rs, rt) => write!(f, "tltu {rs}, {rt}"),
            Instruction::Teq(rs, rt) => write!(f, "teq {rs}, {rt}"),
            Instruction::Tne(rs, rt) => write!(f, "tne {rs}, {rt}"),
            Instruction::Dsll(rd, rt, sa) => write!(f, "{rd} = dsll {rt}, {sa}"),
            Instruction::Dsrl(rd, rt, sa) => write!(f, "{rd} = dsrl {rt}, {sa}"),
            Instruction::Dsra(rd, rt, sa) => write!(f, "{rd} = dsra {rt}, {sa}"),
            Instruction::Dsll32(rd, rt, sa) => write!(f, "{rd} = dsll32 {rt}, {sa}"),
            Instruction::Dsrl32(rd, rt, sa) => write!(f, "{rd} = dsrl32 {rt}, {sa}"),
            Instruction::Dsra32(rd, rt, sa) => write!(f, "{rd} = dsra32 {rt}, {sa}"),
            Instruction::Bltz(rs, imm16) => write!(f, "bltz {rs}, {imm16:#x}"),
            Instruction::Bgez(rs, imm16) => write!(f, "bgez {rs}, {imm16:#x}"),
            Instruction::J(imm26) => write!(f, "j {imm26:#x}"),
            Instruction::Jal(imm26) => write!(f, "jal {imm26:#x}"),
            Instruction::Beq(rs, rt, imm16) => write!(f, "beq {rs}, {rt}, {imm16:#x}"),
            Instruction::Bne(rs, rt, imm16) => write!(f, "bne {rs}, {rt}, {imm16:#x}"),
            Instruction::Blez(rs, imm16) => write!(f, "blez {rs}, {imm16:#x}"),
            Instruction::Addi(rt, rs, imm16) => write!(f, "{rt} = addi {rs}, {imm16}"),
            Instruction::Addiu(rt, rs, imm16) => write!(f, "{rt} = addiu {rs}, {imm16}"),
            Instruction::Slti(rt, rs, imm16) => write!(f, "{rt} = slti {rs}, {imm16}"),
            Instruction::Sltiu(rt, rs, imm16) => write!(f, "{rt} = sltiu {rs}, {imm16}"),
            Instruction::Andi(rt, rs, imm16) => write!(f, "{rt} = andi {rs}, {imm16}"),
            Instruction::Ori(rt, rs, imm16) => write!(f, "{rt} = ori {rs}, {imm16}"),
            Instruction::Xori(rt, rs, imm16) => write!(f, "{rt} = xori {rs}, {imm16}"),
            Instruction::Lui(rt, imm16) => write!(f, "{rt} = lui {imm16:#x}"),
            Instruction::Mfc0(rt, cd) => write!(f, "{rt} = mfc0 {cd}"),
            Instruction::Mtc0(cd, rt) => write!(f, "{cd} = mtc0 {rt}"),
            Instruction::Tlbr => write!(f, "tlbr"),
            Instruction::Tlbwi => write!(f, "tlbwi"),
            Instruction::Tlbwr => write!(f, "tlbwr"),
            Instruction::Tlbp => write!(f, "tlbp"),
            Instruction::Ei => write!(f, "ei"),
            Instruction::Mfc1(rt, fs) => write!(f, "{rt} = mfc1 {fs}"),
            Instruction::Mtc1(fs, rt) => write!(f, "{fs} = mtc1 {rt}"),
            Instruction::Muls(fd, fs, ft) => write!(f, "{fd} = mul.s {fs}, {ft}"),
            Instruction::Divs(fd, fs, ft) => write!(f, "{fd} = div.s {fs}, {ft}"),
            Instruction::Movs(fd, fs) => write!(f, "{fd} = mov.s {fs}"),
            Instruction::Cvtws(fd, fs) => write!(f, "{fd} = cvt.w.s {fs}"),
            Instruction::Cvtsw(fd, fs) => write!(f, "{fd} = cvt.s.w {fs}"),
            Instruction::Beql(rs, rt, imm16) => write!(f, "beql {rs}, {rt}, {imm16:#x}"),
            Instruction::Bnel(rs, rt, imm16) => write!(f, "bnel {rs}, {rt}, {imm16:#x}"),
            Instruction::Mfhi1(rd) => write!(f, "{rd} = mfhi1"),
            Instruction::Mthi1(rs) => write!(f, "mthi1 {rs}"),
            Instruction::Mflo1(rd) => write!(f, "{rd} = mflo1"),
            Instruction::Mtlo1(rs) => write!(f, "mtlo1 {rs}"),
            Instruction::Div1(rs, rt) => write!(f, "div1 {rs}, {rt}"),
            Instruction::Divu1(rs, rt) => write!(f, "divu1 {rs}, {rt}"),
            Instruction::Sq(rt, imm16, rs) => write!(f, "sq {rt}, {imm16:#x}({rs})"),
            Instruction::Lb(rt, imm16, rs) => write!(f, "{rt} = lb {imm16:#x}({rs})"),
            Instruction::Lh(rt, imm16, rs) => write!(f, "{rt} = lh {imm16:#x}({rs})"),
            Instruction::Lw(rt, imm16, rs) => write!(f, "{rt} = lw {imm16:#x}({rs})"),
            Instruction::Lbu(rt, imm16, rs) => write!(f, "{rt} = lbu {imm16:#x}({rs})"),
            Instruction::Lhu(rt, imm16, rs) => write!(f, "{rt} = lhu {imm16:#x}({rs})"),
            Instruction::Lwr(rt, imm16, rs) => write!(f, "{rt} = lwr {imm16:#x}({rs})"),
            Instruction::Sb(rt, imm16, rs) => write!(f, "sb {rt}, {imm16:#x}({rs})"),
            Instruction::Sh(rt, imm16, rs) => write!(f, "sh {rt}, {imm16:#x}({rs})"),
            Instruction::Sw(rt, imm16, rs) => write!(f, "sw {rt}, {imm16:#x}({rs})"),
            Instruction::Lwc1(ft, imm16, rs) => write!(f, "{ft} = lwc1 {imm16:#x}({rs})"),
            Instruction::Ld(rt, imm16, rs) => write!(f, "{rt} = ld {imm16:#x}({rs})"),
            Instruction::Swc1(ft, imm16, rs) => write!(f, "swc1 {ft}, {imm16:#x}({rs})"),
            Instruction::Sd(rt, imm16, rs) => write!(f, "sd {rt}, {imm16:#x}({rs})"),
        }
    }
}

impl Instruction {
    pub fn is_branch(self) -> bool {
        matches!(self, Instruction::Jr(..) | Instruction::Jalr(..) | Instruction::Bltz(..) | Instruction::Bgez(..) | Instruction::J(..) | Instruction::Jal(..) | Instruction::Beq(..) | Instruction::Bne(..) | Instruction::Blez(..) | Instruction::Beql(..) | Instruction::Bnel(..))
    }

    pub fn is_branch_likely(self) -> bool {
        matches!(self, Instruction::Beql(..) | Instruction::Bnel(..))
    }
}

impl Instruction {
    pub fn raw_definitions(self) -> [Option<Occurrence>; 3] {
        match self {
            Instruction::Sll(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Unknown => [None, None, None],
            Instruction::Srl(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Sra(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Sllv(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Srlv(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Srav(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Jr(_) => [None, None, None],
            Instruction::Jalr(_, _) => [None, None, None],
            Instruction::Movz(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Movn(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Syscall => [None, None, None],
            Instruction::Break => [None, None, None],
            Instruction::Sync => [None, None, None],
            Instruction::Mfhi(rd) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mthi(_) => [Some(Occurrence::from(Register::Hi)), None, None],
            Instruction::Mflo(rd) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mtlo(_) => [Some(Occurrence::from(Register::Lo)), None, None],
            Instruction::Dsllv(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsrlv(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsrav(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mult(rd, _, _) => [Some(Occurrence::from(rd)), Some(Occurrence::from(Register::Lo)), Some(Occurrence::from(Register::Hi))],
            Instruction::Multu(rd, _, _) => [Some(Occurrence::from(rd)), Some(Occurrence::from(Register::Lo)), Some(Occurrence::from(Register::Hi))],
            Instruction::Div(_, _) => [Some(Occurrence::from(Register::Lo)), Some(Occurrence::from(Register::Hi)), None],
            Instruction::Divu(_, _) => [Some(Occurrence::from(Register::Lo)), Some(Occurrence::from(Register::Hi)), None],
            Instruction::Add(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Addu(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Sub(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Subu(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::And(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Or(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Xor(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Nor(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mfsa(rd) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mtsa(_) => [None, None, None],
            Instruction::Slt(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Sltu(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dadd(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Daddu(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsub(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsubu(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Tge(_, _) => [None, None, None],
            Instruction::Tgeu(_, _) => [None, None, None],
            Instruction::Tlt(_, _) => [None, None, None],
            Instruction::Tltu(_, _) => [None, None, None],
            Instruction::Teq(_, _) => [None, None, None],
            Instruction::Tne(_, _) => [None, None, None],
            Instruction::Dsll(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsrl(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsra(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsll32(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsrl32(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Dsra32(rd, _, _) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Bltz(_, _) => [None, None, None],
            Instruction::Bgez(_, _) => [None, None, None],
            Instruction::J(_) => [None, None, None],
            Instruction::Jal(_) => [None, None, None],
            Instruction::Beq(_, _, _) => [None, None, None],
            Instruction::Bne(_, _, _) => [None, None, None],
            Instruction::Blez(_, _) => [None, None, None],
            Instruction::Addi(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Addiu(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Slti(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Sltiu(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Andi(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Ori(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Xori(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Lui(rt, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Mfc0(rt, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Mtc0(cd, _) => [Some(Occurrence::from(cd)), None, None],
            Instruction::Tlbr => [None, None, None],
            Instruction::Tlbwi => [None, None, None],
            Instruction::Tlbwr => [None, None, None],
            Instruction::Tlbp => [None, None, None],
            Instruction::Ei => [None, None, None],
            Instruction::Mfc1(rt, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Mtc1(fs, _) => [Some(Occurrence::from(fs)), None, None],
            Instruction::Muls(fd, _, _) => [Some(Occurrence::from(fd)), None, None],
            Instruction::Divs(fd, _, _) => [Some(Occurrence::from(fd)), None, None],
            Instruction::Movs(fd, _) => [Some(Occurrence::from(fd)), None, None],
            Instruction::Cvtws(fd, _) => [Some(Occurrence::from(fd)), None, None],
            Instruction::Cvtsw(fd, _) => [Some(Occurrence::from(fd)), None, None],
            Instruction::Beql(_, _, _) => [None, None, None],
            Instruction::Bnel(_, _, _) => [None, None, None],
            Instruction::Mfhi1(rd) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mthi1(_) => [Some(Occurrence::from(Register::Hi)), None, None],
            Instruction::Mflo1(rd) => [Some(Occurrence::from(rd)), None, None],
            Instruction::Mtlo1(_) => [Some(Occurrence::from(Register::Lo)), None, None],
            Instruction::Div1(_, _) => [Some(Occurrence::from(Register::Lo)), Some(Occurrence::from(Register::Hi)), None],
            Instruction::Divu1(_, _) => [Some(Occurrence::from(Register::Lo)), Some(Occurrence::from(Register::Hi)), None],
            Instruction::Sq(_, _, _) => [None, None, None],
            Instruction::Lb(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Lh(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Lw(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Lbu(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Lhu(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Lwr(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Sb(_, _, _) => [None, None, None],
            Instruction::Sh(_, _, _) => [None, None, None],
            Instruction::Sw(_, _, _) => [None, None, None],
            Instruction::Lwc1(ft, _, _) => [Some(Occurrence::from(ft)), None, None],
            Instruction::Ld(rt, _, _) => [Some(Occurrence::from(rt)), None, None],
            Instruction::Swc1(_, _, _) => [None, None, None],
            Instruction::Sd(_, _, _) => [None, None, None],
        }
    }

    pub fn raw_uses(self) -> [Option<Occurrence>; 2] {
        match self {
            Instruction::Sll(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Unknown => [None, None],
            Instruction::Srl(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Sra(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Sllv(_, rt, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Srlv(_, rt, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Srav(_, rt, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Jr(rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Jalr(rd, rs) => [Some(Occurrence::from(rd)), Some(Occurrence::from(rs))],
            Instruction::Movz(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Movn(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Syscall => [None, None],
            Instruction::Break => [None, None],
            Instruction::Sync => [None, None],
            Instruction::Mfhi(_) => [Some(Occurrence::from(Register::Hi)), None],
            Instruction::Mthi(rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Mflo(_) => [Some(Occurrence::from(Register::Lo)), None],
            Instruction::Mtlo(rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Dsllv(_, rt, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Dsrlv(_, rt, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Dsrav(_, rt, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Mult(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Multu(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Div(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Divu(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Add(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Addu(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Sub(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Subu(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::And(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Or(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Xor(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Nor(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Mfsa(_) => [None, None],
            Instruction::Mtsa(rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Slt(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Sltu(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Dadd(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Daddu(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Dsub(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Dsubu(_, rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Tge(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Tgeu(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Tlt(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Tltu(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Teq(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Tne(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Dsll(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Dsrl(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Dsra(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Dsll32(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Dsrl32(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Dsra32(_, rt, _) => [Some(Occurrence::from(rt)), None],
            Instruction::Bltz(rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Bgez(rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::J(_) => [None, None],
            Instruction::Jal(_) => [None, None],
            Instruction::Beq(rs, rt, _) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Bne(rs, rt, _) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Blez(rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Addi(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Addiu(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Slti(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Sltiu(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Andi(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Ori(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Xori(_, rs, _) => [Some(Occurrence::from(rs)), None],
            Instruction::Lui(_, _) => [None, None],
            Instruction::Mfc0(_, cd) => [Some(Occurrence::from(cd)), None],
            Instruction::Mtc0(_, rt) => [Some(Occurrence::from(rt)), None],
            Instruction::Tlbr => [None, None],
            Instruction::Tlbwi => [None, None],
            Instruction::Tlbwr => [None, None],
            Instruction::Tlbp => [None, None],
            Instruction::Ei => [None, None],
            Instruction::Mfc1(_, fs) => [Some(Occurrence::from(fs)), None],
            Instruction::Mtc1(_, rt) => [Some(Occurrence::from(rt)), None],
            Instruction::Muls(_, fs, ft) => [Some(Occurrence::from(fs)), Some(Occurrence::from(ft))],
            Instruction::Divs(_, fs, ft) => [Some(Occurrence::from(fs)), Some(Occurrence::from(ft))],
            Instruction::Movs(_, fs) => [Some(Occurrence::from(fs)), None],
            Instruction::Cvtws(_, fs) => [Some(Occurrence::from(fs)), None],
            Instruction::Cvtsw(_, fs) => [Some(Occurrence::from(fs)), None],
            Instruction::Beql(rs, rt, _) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Bnel(rs, rt, _) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Mfhi1(_) => [Some(Occurrence::from(Register::Hi)), None],
            Instruction::Mthi1(rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Mflo1(_) => [Some(Occurrence::from(Register::Lo)), None],
            Instruction::Mtlo1(rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Div1(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Divu1(rs, rt) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Sq(rt, _, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Lb(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Lh(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Lw(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Lbu(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Lhu(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Lwr(rt, _, rs) => [Some(Occurrence::from(rs)), Some(Occurrence::from(rt))],
            Instruction::Sb(rt, _, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Sh(rt, _, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Sw(rt, _, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
            Instruction::Lwc1(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Ld(_, _, rs) => [Some(Occurrence::from(rs)), None],
            Instruction::Swc1(ft, _, rs) => [Some(Occurrence::from(ft)), Some(Occurrence::from(rs))],
            Instruction::Sd(rt, _, rs) => [Some(Occurrence::from(rt)), Some(Occurrence::from(rs))],
        }
    }
}
