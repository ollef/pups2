use crate::bits::Bits;

use super::{fpu, instruction::Instruction, register::Register};

pub fn disassemble(data: u32) -> Instruction {
    let opcode = data.bits(26..32);
    let s = data.bits(21..26);
    let t = data.bits(16..21);
    let d = data.bits(11..16);
    let rs = Register::from(s);
    let rt = Register::from(t);
    let rd = Register::from(d);
    let ft = fpu::Register::from(t);
    let fs = fpu::Register::from(d);
    let fd = fpu::Register::from(data.bits(6..11));
    let shamt = data.bits(6..11) as u8;
    let imm16 = data.bits(0..16) as u16;
    let imm26 = data.bits(0..26);
    match opcode {
        0b000000 => match data.bits(0..6) {
            0b000000 => Instruction::Sll(rd, rt, shamt),
            0b000001 => Instruction::Unknown,
            0b000010 => Instruction::Srl(rd, rt, shamt),
            0b000011 => Instruction::Sra(rd, rt, shamt),
            0b000100 => Instruction::Sllv(rd, rt, rs),
            0b000101 => Instruction::Unknown,
            0b000110 => Instruction::Srlv(rd, rt, rs),
            0b000111 => Instruction::Srav(rd, rt, rs),
            0b001000 => Instruction::Jr(rs),
            0b001001 => Instruction::Jalr(rd, rs),
            0b001010 => Instruction::Movz(rd, rs, rt),
            0b001011 => Instruction::Movn(rd, rs, rt),
            0b001100 => Instruction::Syscall,
            0b001101 => Instruction::Break,
            0b001110 => Instruction::Unknown,
            0b001111 => Instruction::Sync,
            0b010000 => Instruction::Mfhi(rd),
            0b010001 => Instruction::Mthi(rs),
            0b010010 => Instruction::Mflo(rd),
            0b010011 => Instruction::Mtlo(rs),
            0b010100 => Instruction::Dsllv(rd, rt, rs),
            0b010101 => Instruction::Unknown,
            0b010110 => Instruction::Dsrlv(rd, rt, rs),
            0b010111 => Instruction::Dsrav(rd, rt, rs),
            0b011000 => Instruction::Mult(rd, rs, rt),
            0b011001 => Instruction::Multu(rd, rs, rt),
            0b011010 => Instruction::Div(rs, rt),
            0b011011 => Instruction::Divu(rs, rt),
            0b011100 => Instruction::Unknown,
            0b011101 => Instruction::Unknown,
            0b011110 => Instruction::Unknown,
            0b011111 => Instruction::Unknown,
            0b100000 => Instruction::Add(rd, rs, rt),
            0b100001 => Instruction::Addu(rd, rs, rt),
            0b100010 => Instruction::Sub(rd, rs, rt),
            0b100011 => Instruction::Subu(rd, rs, rt),
            0b100100 => Instruction::And(rd, rs, rt),
            0b100101 => Instruction::Or(rd, rs, rt),
            0b100110 => Instruction::Xor(rd, rs, rt),
            0b100111 => Instruction::Nor(rd, rs, rt),
            0b101000 => Instruction::Mfsa(rd),
            0b101001 => Instruction::Mtsa(rs),
            0b101010 => Instruction::Slt(rd, rs, rt),
            0b101011 => Instruction::Sltu(rd, rs, rt),
            0b101100 => Instruction::Dadd(rd, rs, rt),
            0b101101 => Instruction::Daddu(rd, rs, rt),
            0b101110 => Instruction::Dsub(rd, rs, rt),
            0b101111 => Instruction::Dsubu(rd, rs, rt),
            0b110000 => Instruction::Tge(rs, rt),
            0b110001 => Instruction::Tgeu(rs, rt),
            0b110010 => Instruction::Tlt(rs, rt),
            0b110011 => Instruction::Tltu(rs, rt),
            0b110100 => Instruction::Teq(rs, rt),
            0b110101 => Instruction::Unknown,
            0b110110 => Instruction::Tne(rs, rt),
            0b110111 => Instruction::Unknown,
            0b111000 => Instruction::Dsll(rd, rt, shamt),
            0b111001 => Instruction::Unknown,
            0b111010 => Instruction::Dsrl(rd, rt, shamt),
            0b111011 => Instruction::Dsra(rd, rt, shamt),
            0b111100 => Instruction::Dsll32(rd, rt, shamt),
            0b111101 => Instruction::Unknown,
            0b111110 => Instruction::Dsrl32(rd, rt, shamt),
            0b111111 => Instruction::Dsra32(rd, rt, shamt),
            _ => panic!("Special not implemented {:#034b}", data),
        },
        0b000001 => match t {
            0b00000 => Instruction::Bltz(rs, imm16),
            0b00001 => Instruction::Bgez(rs, imm16),
            _ => panic!("Branch not implemented t={:#05b} {:#034b}", t, data),
        },
        0b000010 => Instruction::J(imm26),
        0b000011 => Instruction::Jal(imm26),
        0b000100 => Instruction::Beq(rs, rt, imm16),
        0b000101 => Instruction::Bne(rs, rt, imm16),
        0b001001 => Instruction::Addiu(rt, rs, imm16),
        0b001011 => Instruction::Sltiu(rt, rs, imm16),
        0b001100 => Instruction::Andi(rt, rs, imm16),
        0b001101 => Instruction::Ori(rt, rs, imm16),
        0b001111 => Instruction::Lui(rt, imm16),
        0b010000 => match s {
            0b10000 => match data.bits(0..6) {
                0b111000 => Instruction::Ei,
                _ => panic!("TLB/Exception not implemented {:#034b}", data),
            },
            _ => panic!("COP0 not implemented s={:05b} {:#034b}", s, data),
        },
        0b010001 => match (s, data.bits(0..6)) {
            (0b00100, _) => Instruction::Mtc1(rt, fs),
            (0b10000, 0b000010) => Instruction::Muls(fd, fs, ft),
            (0b10000, 0b000011) => Instruction::Divs(fd, fs, ft),
            (0b10000, 0b000110) => Instruction::Movs(fd, fs),
            (0b10100, 0b100000) => Instruction::Cvtsw(fd, fs),
            _ => panic!(
                "COP1 not implemented s={:05b} {:06b} {:#034b}",
                s,
                data.bits(0..6),
                data
            ),
        },
        0b010100 => Instruction::Beql(rs, rt, imm16),
        0b011111 => Instruction::Sq(rt, rs, imm16),
        0b100001 => Instruction::Lh(rt, rs, imm16),
        0b100011 => Instruction::Lw(rt, rs, imm16),
        0b100100 => Instruction::Lbu(rt, rs, imm16),
        0b100101 => Instruction::Lhu(rt, rs, imm16),
        0b100110 => Instruction::Lwr(rt, rs, imm16),
        0b101000 => Instruction::Sb(rt, rs, imm16),
        0b101001 => Instruction::Sh(rt, rs, imm16),
        0b101011 => Instruction::Sw(rt, rs, imm16),
        0b110001 => Instruction::Lwc1(ft, rs, imm16),
        0b110111 => Instruction::Ld(rt, rs, imm16),
        0b111001 => Instruction::Swc1(ft, rs, imm16),
        0b111111 => Instruction::Sd(rt, rs, imm16),
        _ => panic!("Not implemented opcode={:06b}, {:#034b}", opcode, data),
    }
}
