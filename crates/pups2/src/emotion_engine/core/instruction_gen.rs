use super::control;
use super::fpu;
use super::instruction::{case, opcode_pattern, Definition, Instruction, Use};
use super::register::Register;
use crate::bits::Bits;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Opcode {
    Sll,
    Unknown,
    Srl,
    Sra,
    Sllv,
    Srlv,
    Srav,
    Jr,
    Jalr,
    Movz,
    Movn,
    Syscall,
    Break,
    Sync,
    Mfhi,
    Mthi,
    Mflo,
    Mtlo,
    Dsllv,
    Dsrlv,
    Dsrav,
    Mult,
    Multu,
    Div,
    Divu,
    Add,
    Addu,
    Sub,
    Subu,
    And,
    Or,
    Xor,
    Nor,
    Mfsa,
    Mtsa,
    Slt,
    Sltu,
    Dadd,
    Daddu,
    Dsub,
    Dsubu,
    Tge,
    Tgeu,
    Tlt,
    Tltu,
    Teq,
    Tne,
    Dsll,
    Dsrl,
    Dsra,
    Dsll32,
    Dsrl32,
    Dsra32,
    Bltz,
    Bgez,
    J,
    Jal,
    Beq,
    Bne,
    Blez,
    Addi,
    Addiu,
    Slti,
    Sltiu,
    Andi,
    Ori,
    Xori,
    Lui,
    Mfc0,
    Mtc0,
    Tlbr,
    Tlbwi,
    Tlbwr,
    Tlbp,
    Ei,
    Mfc1,
    Mtc1,
    Muls,
    Divs,
    Movs,
    Cvtws,
    Cvtsw,
    Beql,
    Bnel,
    Sq,
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
    Lwr,
    Sb,
    Sh,
    Sw,
    Lwc1,
    Ld,
    Swc1,
    Sd,
}

impl Opcode {
    pub fn decode(data: u32) -> Opcode {
        match data.bits(26..32) {
            0b000000 => match data.bits(0..6) {
                0b000000 => Opcode::Sll,
                0b000001 => Opcode::Unknown,
                0b000010 => Opcode::Srl,
                0b000011 => Opcode::Sra,
                0b000100 => Opcode::Sllv,
                0b000101 => Opcode::Unknown,
                0b000110 => Opcode::Srlv,
                0b000111 => Opcode::Srav,
                0b001000 => Opcode::Jr,
                0b001001 => Opcode::Jalr,
                0b001010 => Opcode::Movz,
                0b001011 => Opcode::Movn,
                0b001100 => Opcode::Syscall,
                0b001101 => Opcode::Break,
                0b001110 => Opcode::Unknown,
                0b001111 => Opcode::Sync,
                0b010000 => Opcode::Mfhi,
                0b010001 => Opcode::Mthi,
                0b010010 => Opcode::Mflo,
                0b010011 => Opcode::Mtlo,
                0b010100 => Opcode::Dsllv,
                0b010101 => Opcode::Unknown,
                0b010110 => Opcode::Dsrlv,
                0b010111 => Opcode::Dsrav,
                0b011000 => Opcode::Mult,
                0b011001 => Opcode::Multu,
                0b011010 => Opcode::Div,
                0b011011 => Opcode::Divu,
                0b011100 => Opcode::Unknown,
                0b011101 => Opcode::Unknown,
                0b011110 => Opcode::Unknown,
                0b011111 => Opcode::Unknown,
                0b100000 => Opcode::Add,
                0b100001 => Opcode::Addu,
                0b100010 => Opcode::Sub,
                0b100011 => Opcode::Subu,
                0b100100 => Opcode::And,
                0b100101 => Opcode::Or,
                0b100110 => Opcode::Xor,
                0b100111 => Opcode::Nor,
                0b101000 => Opcode::Mfsa,
                0b101001 => Opcode::Mtsa,
                0b101010 => Opcode::Slt,
                0b101011 => Opcode::Sltu,
                0b101100 => Opcode::Dadd,
                0b101101 => Opcode::Daddu,
                0b101110 => Opcode::Dsub,
                0b101111 => Opcode::Dsubu,
                0b110000 => Opcode::Tge,
                0b110001 => Opcode::Tgeu,
                0b110010 => Opcode::Tlt,
                0b110011 => Opcode::Tltu,
                0b110100 => Opcode::Teq,
                0b110101 => Opcode::Unknown,
                0b110110 => Opcode::Tne,
                0b110111 => Opcode::Unknown,
                0b111000 => Opcode::Dsll,
                0b111001 => Opcode::Unknown,
                0b111010 => Opcode::Dsrl,
                0b111011 => Opcode::Dsra,
                0b111100 => Opcode::Dsll32,
                0b111101 => Opcode::Unknown,
                0b111110 => Opcode::Dsrl32,
                0b111111 => Opcode::Dsra32,
                _ => unreachable!(),
            }
            0b000001 => match data.bits(16..21) {
                0b00000 => Opcode::Bltz,
                0b00001 => Opcode::Bgez,
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b000010 => Opcode::J,
            0b000011 => Opcode::Jal,
            0b000100 => Opcode::Beq,
            0b000101 => Opcode::Bne,
            0b000110 => match data.bits(16..21) {
                0b00000 => Opcode::Blez,
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b001000 => Opcode::Addi,
            0b001001 => Opcode::Addiu,
            0b001010 => Opcode::Slti,
            0b001011 => Opcode::Sltiu,
            0b001100 => Opcode::Andi,
            0b001101 => Opcode::Ori,
            0b001110 => Opcode::Xori,
            0b001111 => match data.bits(21..26) {
                0b00000 => Opcode::Lui,
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b010000 => match data.bits(0..11) {
                0b00000000000 => match data.bits(21..26) {
                    0b00000 => Opcode::Mfc0,
                    0b00100 => Opcode::Mtc0,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000000001 => match data.bits(11..26) {
                    0b100000000000000 => Opcode::Tlbr,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000000010 => match data.bits(11..26) {
                    0b100000000000000 => Opcode::Tlbwi,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000000110 => match data.bits(11..26) {
                    0b100000000000000 => Opcode::Tlbwr,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000001000 => match data.bits(11..26) {
                    0b100000000000000 => Opcode::Tlbp,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b00000111000 => match data.bits(11..26) {
                    0b100000000000000 => Opcode::Ei,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b010001 => match data.bits(0..6) {
                0b000000 => match data.bits(6..11) {
                    0b00000 => match data.bits(21..26) {
                        0b00000 => Opcode::Mfc1,
                        0b00100 => Opcode::Mtc1,
                        _ => panic!("Unhandled instruction: {:#034b}", data),
                    }
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000010 => match data.bits(21..26) {
                    0b10000 => Opcode::Muls,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000011 => match data.bits(21..26) {
                    0b10000 => Opcode::Divs,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b000110 => match data.bits(16..26) {
                    0b1000000000 => Opcode::Movs,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100000 => match data.bits(16..26) {
                    0b1010000000 => Opcode::Cvtsw,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                0b100100 => match data.bits(16..26) {
                    0b1000000000 => Opcode::Cvtws,
                    _ => panic!("Unhandled instruction: {:#034b}", data),
                }
                _ => panic!("Unhandled instruction: {:#034b}", data),
            }
            0b010100 => Opcode::Beql,
            0b010101 => Opcode::Bnel,
            0b011111 => Opcode::Sq,
            0b100000 => Opcode::Lb,
            0b100001 => Opcode::Lh,
            0b100011 => Opcode::Lw,
            0b100100 => Opcode::Lbu,
            0b100101 => Opcode::Lhu,
            0b100110 => Opcode::Lwr,
            0b101000 => Opcode::Sb,
            0b101001 => Opcode::Sh,
            0b101011 => Opcode::Sw,
            0b110001 => Opcode::Lwc1,
            0b110111 => Opcode::Ld,
            0b111001 => Opcode::Swc1,
            0b111111 => Opcode::Sd,
            _ => panic!("Unhandled instruction: {:#034b}", data),
        }
    }
}

macro_rules! let_operands {
    ($operands:pat_param, Sll, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    (, Unknown, $raw:expr) => {
    };
    ($operands:pat_param, Srl, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Sra, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Sllv, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Srlv, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Srav, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Jr, $raw:expr) => {
        let $operands = Register::from($raw.bits(21..26));
    };
    ($operands:pat_param, Jalr, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Movz, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Movn, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    (, Syscall, $raw:expr) => {
    };
    (, Break, $raw:expr) => {
    };
    (, Sync, $raw:expr) => {
    };
    ($operands:pat_param, Mfhi, $raw:expr) => {
        let $operands = Register::from($raw.bits(11..16));
    };
    ($operands:pat_param, Mthi, $raw:expr) => {
        let $operands = Register::from($raw.bits(21..26));
    };
    ($operands:pat_param, Mflo, $raw:expr) => {
        let $operands = Register::from($raw.bits(11..16));
    };
    ($operands:pat_param, Mtlo, $raw:expr) => {
        let $operands = Register::from($raw.bits(21..26));
    };
    ($operands:pat_param, Dsllv, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Dsrlv, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Dsrav, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Mult, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Multu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Div, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Divu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Add, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Addu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Sub, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Subu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, And, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Or, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Xor, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Nor, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Mfsa, $raw:expr) => {
        let $operands = Register::from($raw.bits(11..16));
    };
    ($operands:pat_param, Mtsa, $raw:expr) => {
        let $operands = Register::from($raw.bits(21..26));
    };
    ($operands:pat_param, Slt, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Sltu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Dadd, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Daddu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Dsub, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Dsubu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Tge, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Tgeu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Tlt, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Tltu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Teq, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Tne, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Dsll, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Dsrl, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Dsra, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Dsll32, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Dsrl32, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Dsra32, $raw:expr) => {
        let $operands = (Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)), $raw.bits(6..11) as u8);
    };
    ($operands:pat_param, Bltz, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Bgez, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, J, $raw:expr) => {
        let $operands = $raw.bits(0..26);
    };
    ($operands:pat_param, Jal, $raw:expr) => {
        let $operands = $raw.bits(0..26);
    };
    ($operands:pat_param, Beq, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Bne, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Blez, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Addi, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Addiu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Slti, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Sltiu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Andi, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Ori, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Xori, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), Register::from($raw.bits(21..26)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Lui, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Mfc0, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), control::Register::from($raw.bits(11..16)));
    };
    ($operands:pat_param, Mtc0, $raw:expr) => {
        let $operands = (control::Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)));
    };
    (, Tlbr, $raw:expr) => {
    };
    (, Tlbwi, $raw:expr) => {
    };
    (, Tlbwr, $raw:expr) => {
    };
    (, Tlbp, $raw:expr) => {
    };
    (, Ei, $raw:expr) => {
    };
    ($operands:pat_param, Mfc1, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), fpu::Register::from($raw.bits(11..16)));
    };
    ($operands:pat_param, Mtc1, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(11..16)), Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Muls, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(6..11)), fpu::Register::from($raw.bits(11..16)), fpu::Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Divs, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(6..11)), fpu::Register::from($raw.bits(11..16)), fpu::Register::from($raw.bits(16..21)));
    };
    ($operands:pat_param, Movs, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(6..11)), fpu::Register::from($raw.bits(11..16)));
    };
    ($operands:pat_param, Cvtws, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(6..11)), fpu::Register::from($raw.bits(11..16)));
    };
    ($operands:pat_param, Cvtsw, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(6..11)), fpu::Register::from($raw.bits(11..16)));
    };
    ($operands:pat_param, Beql, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Bnel, $raw:expr) => {
        let $operands = (Register::from($raw.bits(21..26)), Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16);
    };
    ($operands:pat_param, Sq, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lb, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lh, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lw, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lbu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lhu, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lwr, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Sb, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Sh, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Sw, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Lwc1, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Ld, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Swc1, $raw:expr) => {
        let $operands = (fpu::Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    ($operands:pat_param, Sd, $raw:expr) => {
        let $operands = (Register::from($raw.bits(16..21)), $raw.bits(0..16) as u16, Register::from($raw.bits(21..26)));
    };
    (, _, $raw:expr) => {};
}
pub(super) use let_operands;

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        case! { self,
            Sll(rd, rt, sa) => write!(f, "{rd} = sll {rt}, {sa}"),
            Unknown => write!(f, "unknown"),
            Srl(rd, rt, sa) => write!(f, "{rd} = srl {rt}, {sa}"),
            Sra(rd, rt, sa) => write!(f, "{rd} = sra {rt}, {sa}"),
            Sllv(rd, rt, rs) => write!(f, "{rd} = sllv {rt}, {rs}"),
            Srlv(rd, rt, rs) => write!(f, "{rd} = srlv {rt}, {rs}"),
            Srav(rd, rt, rs) => write!(f, "{rd} = srav {rt}, {rs}"),
            Jr rs => write!(f, "jr {rs}"),
            Jalr(rd, rs) => write!(f, "jalr {rd}, {rs}"),
            Movz(rd, rs, rt) => write!(f, "{rd} = movz {rs}, {rt}"),
            Movn(rd, rs, rt) => write!(f, "{rd} = movn {rs}, {rt}"),
            Syscall => write!(f, "syscall"),
            Break => write!(f, "break"),
            Sync => write!(f, "sync"),
            Mfhi rd => write!(f, "{rd} = mfhi"),
            Mthi rs => write!(f, "mthi {rs}"),
            Mflo rd => write!(f, "{rd} = mflo"),
            Mtlo rs => write!(f, "mtlo {rs}"),
            Dsllv(rd, rt, rs) => write!(f, "{rd} = dsllv {rt}, {rs}"),
            Dsrlv(rd, rt, rs) => write!(f, "{rd} = dsrlv {rt}, {rs}"),
            Dsrav(rd, rt, rs) => write!(f, "{rd} = dsrav {rt}, {rs}"),
            Mult(rd, rs, rt) => write!(f, "{rd} = mult {rs}, {rt}"),
            Multu(rd, rs, rt) => write!(f, "{rd} = multu {rs}, {rt}"),
            Div(rs, rt) => write!(f, "div {rs}, {rt}"),
            Divu(rs, rt) => write!(f, "divu {rs}, {rt}"),
            Add(rd, rs, rt) => write!(f, "{rd} = add {rs}, {rt}"),
            Addu(rd, rs, rt) => write!(f, "{rd} = addu {rs}, {rt}"),
            Sub(rd, rs, rt) => write!(f, "{rd} = sub {rs}, {rt}"),
            Subu(rd, rs, rt) => write!(f, "{rd} = subu {rs}, {rt}"),
            And(rd, rs, rt) => write!(f, "{rd} = and {rs}, {rt}"),
            Or(rd, rs, rt) => write!(f, "{rd} = or {rs}, {rt}"),
            Xor(rd, rs, rt) => write!(f, "{rd} = xor {rs}, {rt}"),
            Nor(rd, rs, rt) => write!(f, "{rd} = nor {rs}, {rt}"),
            Mfsa rd => write!(f, "{rd} = mfsa"),
            Mtsa rs => write!(f, "mtsa {rs}"),
            Slt(rd, rs, rt) => write!(f, "{rd} = slt {rs}, {rt}"),
            Sltu(rd, rs, rt) => write!(f, "{rd} = sltu {rs}, {rt}"),
            Dadd(rd, rs, rt) => write!(f, "{rd} = dadd {rs}, {rt}"),
            Daddu(rd, rs, rt) => write!(f, "{rd} = daddu {rs}, {rt}"),
            Dsub(rd, rs, rt) => write!(f, "{rd} = dsub {rs}, {rt}"),
            Dsubu(rd, rs, rt) => write!(f, "{rd} = dsubu {rs}, {rt}"),
            Tge(rs, rt) => write!(f, "tge {rs}, {rt}"),
            Tgeu(rs, rt) => write!(f, "tgeu {rs}, {rt}"),
            Tlt(rs, rt) => write!(f, "tlt {rs}, {rt}"),
            Tltu(rs, rt) => write!(f, "tltu {rs}, {rt}"),
            Teq(rs, rt) => write!(f, "teq {rs}, {rt}"),
            Tne(rs, rt) => write!(f, "tne {rs}, {rt}"),
            Dsll(rd, rt, sa) => write!(f, "{rd} = dsll {rt}, {sa}"),
            Dsrl(rd, rt, sa) => write!(f, "{rd} = dsrl {rt}, {sa}"),
            Dsra(rd, rt, sa) => write!(f, "{rd} = dsra {rt}, {sa}"),
            Dsll32(rd, rt, sa) => write!(f, "{rd} = dsll32 {rt}, {sa}"),
            Dsrl32(rd, rt, sa) => write!(f, "{rd} = dsrl32 {rt}, {sa}"),
            Dsra32(rd, rt, sa) => write!(f, "{rd} = dsra32 {rt}, {sa}"),
            Bltz(rs, imm16) => write!(f, "bltz {rs}, {imm16}"),
            Bgez(rs, imm16) => write!(f, "bgez {rs}, {imm16}"),
            J imm26 => write!(f, "j {imm26}"),
            Jal imm26 => write!(f, "jal {imm26}"),
            Beq(rs, rt, imm16) => write!(f, "beq {rs}, {rt}, {imm16}"),
            Bne(rs, rt, imm16) => write!(f, "bne {rs}, {rt}, {imm16}"),
            Blez(rs, imm16) => write!(f, "blez {rs}, {imm16}"),
            Addi(rt, rs, imm16) => write!(f, "{rt} = addi {rs}, {imm16}"),
            Addiu(rt, rs, imm16) => write!(f, "{rt} = addiu {rs}, {imm16}"),
            Slti(rt, rs, imm16) => write!(f, "{rt} = slti {rs}, {imm16}"),
            Sltiu(rt, rs, imm16) => write!(f, "{rt} = sltiu {rs}, {imm16}"),
            Andi(rt, rs, imm16) => write!(f, "{rt} = andi {rs}, {imm16}"),
            Ori(rt, rs, imm16) => write!(f, "{rt} = ori {rs}, {imm16}"),
            Xori(rt, rs, imm16) => write!(f, "{rt} = xori {rs}, {imm16}"),
            Lui(rt, imm16) => write!(f, "{rt} = lui {imm16}"),
            Mfc0(rt, cd) => write!(f, "{rt} = mfc0 {cd}"),
            Mtc0(cd, rt) => write!(f, "{cd} = mtc0 {rt}"),
            Tlbr => write!(f, "tlbr"),
            Tlbwi => write!(f, "tlbwi"),
            Tlbwr => write!(f, "tlbwr"),
            Tlbp => write!(f, "tlbp"),
            Ei => write!(f, "ei"),
            Mfc1(rt, fs) => write!(f, "{rt} = mfc1 {fs}"),
            Mtc1(fs, rt) => write!(f, "{fs} = mtc1 {rt}"),
            Muls(fd, fs, ft) => write!(f, "{fd} = mul.s {fs}, {ft}"),
            Divs(fd, fs, ft) => write!(f, "{fd} = div.s {fs}, {ft}"),
            Movs(fd, fs) => write!(f, "{fd} = mov.s {fs}"),
            Cvtws(fd, fs) => write!(f, "{fd} = cvt.w.s {fs}"),
            Cvtsw(fd, fs) => write!(f, "{fd} = cvt.s.w {fs}"),
            Beql(rs, rt, imm16) => write!(f, "beql {rs}, {rt}, {imm16}"),
            Bnel(rs, rt, imm16) => write!(f, "bnel {rs}, {rt}, {imm16}"),
            Sq(rt, imm16, rs) => write!(f, "sq {rt}, {imm16:#x}({rs})"),
            Lb(rt, imm16, rs) => write!(f, "{rt} = lb {imm16:#x}({rs})"),
            Lh(rt, imm16, rs) => write!(f, "{rt} = lh {imm16:#x}({rs})"),
            Lw(rt, imm16, rs) => write!(f, "{rt} = lw {imm16:#x}({rs})"),
            Lbu(rt, imm16, rs) => write!(f, "{rt} = lbu {imm16:#x}({rs})"),
            Lhu(rt, imm16, rs) => write!(f, "{rt} = lhu {imm16:#x}({rs})"),
            Lwr(rt, imm16, rs) => write!(f, "{rt} = lwr {imm16:#x}({rs})"),
            Sb(rt, imm16, rs) => write!(f, "sb {rt}, {imm16:#x}({rs})"),
            Sh(rt, imm16, rs) => write!(f, "sh {rt}, {imm16:#x}({rs})"),
            Sw(rt, imm16, rs) => write!(f, "sw {rt}, {imm16:#x}({rs})"),
            Lwc1(ft, imm16, rs) => write!(f, "{ft} = lwc1 {imm16:#x}({rs})"),
            Ld(rt, imm16, rs) => write!(f, "{rt} = ld {imm16:#x}({rs})"),
            Swc1(ft, imm16, rs) => write!(f, "swc1 {ft}, {imm16:#x}({rs})"),
            Sd(rt, imm16, rs) => write!(f, "sd {rt}, {imm16:#x}({rs})"),
        }
    }
}

impl Opcode {
    pub fn is_branch_likely(self) -> bool {
        match self {
            Opcode::Beql => true,
            Opcode::Bnel => true,
            _ => false,
        }
    }

    pub fn is_branch(self) -> bool {
        match self {
            Opcode::Jr => true,
            Opcode::Jalr => true,
            Opcode::Bltz => true,
            Opcode::Bgez => true,
            Opcode::J => true,
            Opcode::Jal => true,
            Opcode::Beq => true,
            Opcode::Bne => true,
            Opcode::Blez => true,
            Opcode::Beql => true,
            Opcode::Bnel => true,
            _ => false,
        }
    }
}

impl Instruction {
    fn raw_definitions(self) -> [Option<Definition>; 3] {
        case! { self,
            Sll(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Unknown => [None, None, None],
            Srl(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Sra(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Sllv(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Srlv(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Srav(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Jr _ => [None, None, None],
            Jalr(_, _) => [None, None, None],
            Movz(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Movn(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Syscall => [None, None, None],
            Break => [None, None, None],
            Sync => [None, None, None],
            Mfhi rd => [Some(Definition::from(rd)), None, None],
            Mthi _ => [Some(Definition::from(Register::Hi)), None, None],
            Mflo rd => [Some(Definition::from(rd)), None, None],
            Mtlo _ => [Some(Definition::from(Register::Lo)), None, None],
            Dsllv(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsrlv(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsrav(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Mult(rd, _, _) => [Some(Definition::from(rd)), Some(Definition::from(Register::Lo)), Some(Definition::from(Register::Hi))],
            Multu(rd, _, _) => [Some(Definition::from(rd)), Some(Definition::from(Register::Lo)), Some(Definition::from(Register::Hi))],
            Div(_, _) => [Some(Definition::from(Register::Lo)), Some(Definition::from(Register::Hi)), None],
            Divu(_, _) => [Some(Definition::from(Register::Lo)), Some(Definition::from(Register::Hi)), None],
            Add(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Addu(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Sub(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Subu(rd, _, _) => [Some(Definition::from(rd)), None, None],
            And(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Or(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Xor(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Nor(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Mfsa rd => [Some(Definition::from(rd)), None, None],
            Mtsa _ => [None, None, None],
            Slt(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Sltu(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dadd(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Daddu(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsub(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsubu(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Tge(_, _) => [None, None, None],
            Tgeu(_, _) => [None, None, None],
            Tlt(_, _) => [None, None, None],
            Tltu(_, _) => [None, None, None],
            Teq(_, _) => [None, None, None],
            Tne(_, _) => [None, None, None],
            Dsll(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsrl(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsra(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsll32(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsrl32(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Dsra32(rd, _, _) => [Some(Definition::from(rd)), None, None],
            Bltz(_, _) => [None, None, None],
            Bgez(_, _) => [None, None, None],
            J _ => [None, None, None],
            Jal _ => [None, None, None],
            Beq(_, _, _) => [None, None, None],
            Bne(_, _, _) => [None, None, None],
            Blez(_, _) => [None, None, None],
            Addi(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Addiu(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Slti(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Sltiu(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Andi(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Ori(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Xori(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Lui(rt, _) => [Some(Definition::from(rt)), None, None],
            Mfc0(rt, _) => [Some(Definition::from(rt)), None, None],
            Mtc0(cd, _) => [Some(Definition::from(cd)), None, None],
            Tlbr => [None, None, None],
            Tlbwi => [None, None, None],
            Tlbwr => [None, None, None],
            Tlbp => [None, None, None],
            Ei => [None, None, None],
            Mfc1(rt, _) => [Some(Definition::from(rt)), None, None],
            Mtc1(fs, _) => [Some(Definition::from(fs)), None, None],
            Muls(fd, _, _) => [Some(Definition::from(fd)), None, None],
            Divs(fd, _, _) => [Some(Definition::from(fd)), None, None],
            Movs(fd, _) => [Some(Definition::from(fd)), None, None],
            Cvtws(fd, _) => [Some(Definition::from(fd)), None, None],
            Cvtsw(fd, _) => [Some(Definition::from(fd)), None, None],
            Beql(_, _, _) => [None, None, None],
            Bnel(_, _, _) => [None, None, None],
            Sq(_, _, _) => [None, None, None],
            Lb(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Lh(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Lw(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Lbu(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Lhu(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Lwr(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Sb(_, _, _) => [None, None, None],
            Sh(_, _, _) => [None, None, None],
            Sw(_, _, _) => [None, None, None],
            Lwc1(ft, _, _) => [Some(Definition::from(ft)), None, None],
            Ld(rt, _, _) => [Some(Definition::from(rt)), None, None],
            Swc1(_, _, _) => [None, None, None],
            Sd(_, _, _) => [None, None, None],
        }
    }

    fn raw_uses(self) -> [Option<Use>; 2] {
        case! { self,
            Sll(_, rt, _) => [Some(Use::from(rt)), None],
            Unknown => [None, None],
            Srl(_, rt, _) => [Some(Use::from(rt)), None],
            Sra(_, rt, _) => [Some(Use::from(rt)), None],
            Sllv(_, rt, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Srlv(_, rt, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Srav(_, rt, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Jr rs => [Some(Use::from(rs)), None],
            Jalr(rd, rs) => [Some(Use::from(rd)), Some(Use::from(rs))],
            Movz(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Movn(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Syscall => [None, None],
            Break => [None, None],
            Sync => [None, None],
            Mfhi _ => [Some(Use::from(Register::Hi)), None],
            Mthi rs => [Some(Use::from(rs)), None],
            Mflo _ => [Some(Use::from(Register::Lo)), None],
            Mtlo rs => [Some(Use::from(rs)), None],
            Dsllv(_, rt, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Dsrlv(_, rt, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Dsrav(_, rt, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Mult(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Multu(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Div(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Divu(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Add(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Addu(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Sub(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Subu(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            And(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Or(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Xor(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Nor(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Mfsa _ => [None, None],
            Mtsa rs => [Some(Use::from(rs)), None],
            Slt(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Sltu(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Dadd(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Daddu(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Dsub(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Dsubu(_, rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Tge(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Tgeu(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Tlt(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Tltu(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Teq(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Tne(rs, rt) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Dsll(_, rt, _) => [Some(Use::from(rt)), None],
            Dsrl(_, rt, _) => [Some(Use::from(rt)), None],
            Dsra(_, rt, _) => [Some(Use::from(rt)), None],
            Dsll32(_, rt, _) => [Some(Use::from(rt)), None],
            Dsrl32(_, rt, _) => [Some(Use::from(rt)), None],
            Dsra32(_, rt, _) => [Some(Use::from(rt)), None],
            Bltz(rs, _) => [Some(Use::from(rs)), None],
            Bgez(rs, _) => [Some(Use::from(rs)), None],
            J _ => [None, None],
            Jal _ => [None, None],
            Beq(rs, rt, _) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Bne(rs, rt, _) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Blez(rs, _) => [Some(Use::from(rs)), None],
            Addi(_, rs, _) => [Some(Use::from(rs)), None],
            Addiu(_, rs, _) => [Some(Use::from(rs)), None],
            Slti(_, rs, _) => [Some(Use::from(rs)), None],
            Sltiu(_, rs, _) => [Some(Use::from(rs)), None],
            Andi(_, rs, _) => [Some(Use::from(rs)), None],
            Ori(_, rs, _) => [Some(Use::from(rs)), None],
            Xori(_, rs, _) => [Some(Use::from(rs)), None],
            Lui(_, _) => [None, None],
            Mfc0(_, cd) => [Some(Use::from(cd)), None],
            Mtc0(_, rt) => [Some(Use::from(rt)), None],
            Tlbr => [None, None],
            Tlbwi => [None, None],
            Tlbwr => [None, None],
            Tlbp => [None, None],
            Ei => [None, None],
            Mfc1(_, fs) => [Some(Use::from(fs)), None],
            Mtc1(_, rt) => [Some(Use::from(rt)), None],
            Muls(_, fs, ft) => [Some(Use::from(fs)), Some(Use::from(ft))],
            Divs(_, fs, ft) => [Some(Use::from(fs)), Some(Use::from(ft))],
            Movs(_, fs) => [Some(Use::from(fs)), None],
            Cvtws(_, fs) => [Some(Use::from(fs)), None],
            Cvtsw(_, fs) => [Some(Use::from(fs)), None],
            Beql(rs, rt, _) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Bnel(rs, rt, _) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Sq(rt, _, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Lb(_, _, rs) => [Some(Use::from(rs)), None],
            Lh(_, _, rs) => [Some(Use::from(rs)), None],
            Lw(_, _, rs) => [Some(Use::from(rs)), None],
            Lbu(_, _, rs) => [Some(Use::from(rs)), None],
            Lhu(_, _, rs) => [Some(Use::from(rs)), None],
            Lwr(rt, _, rs) => [Some(Use::from(rs)), Some(Use::from(rt))],
            Sb(rt, _, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Sh(rt, _, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Sw(rt, _, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
            Lwc1(_, _, rs) => [Some(Use::from(rs)), None],
            Ld(_, _, rs) => [Some(Use::from(rs)), None],
            Swc1(ft, _, rs) => [Some(Use::from(ft)), Some(Use::from(rs))],
            Sd(rt, _, rs) => [Some(Use::from(rt)), Some(Use::from(rs))],
        }
    }
}

