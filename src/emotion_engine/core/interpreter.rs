use crate::{
    bits::{Bits, SignExtend},
    emotion_engine::{
        bus::Bus,
        core::register::{AnyRegister, Register},
    },
};

use super::{instruction::Instruction, Core};

impl Core {
    pub fn set_delayed_branch_target(&mut self, target: u32) {
        if target.bits(0..2) != 0 {
            panic!("Invalid branch target: {:#010x}", target);
        }
        self.delayed_branch_target = Some(target);
    }

    pub fn interpret_instruction(&mut self, instruction: Instruction, bus: &mut Bus) {
        // for reg in instruction.uses() {
        //     let value = self.get_register::<u64>(reg);
        //     println!("{}={:#x}", reg, value);
        // }
        let mut next_program_counter = self
            .delayed_branch_target
            .take()
            .unwrap_or(self.program_counter + 4);
        // println!("pc={:#010}: {instruction}", self.program_counter);
        match instruction {
            Instruction::Unknown => {
                println!("Unknown instruction at {:#010x}", self.program_counter)
            }
            Instruction::Sll(rd, rt, shamt) => {
                let value = self.get_register::<u32>(rt) << shamt;
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Srl(rd, rt, shamt) => {
                let value = self.get_register::<u32>(rt) >> shamt;
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Sra(rd, rt, shamt) => {
                let value = (self.get_register::<u32>(rt) as i32) >> shamt;
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Sllv(rd, rt, rs) => {
                let value = self.get_register::<u32>(rt) << self.get_register::<u32>(rs).bits(0..5);
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Srlv(rd, rt, rs) => {
                let value = self.get_register::<u32>(rt) >> self.get_register::<u32>(rs).bits(0..5);
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Srav(rd, rt, rs) => {
                let value = (self.get_register::<u32>(rt) as i32)
                    >> self.get_register::<u32>(rs).bits(0..5);
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Jr(rs) => {
                self.set_delayed_branch_target(self.get_register::<u32>(rs));
            }
            Instruction::Jalr(rd, rs) => {
                let branch_target = self.get_register::<u32>(rs);
                self.set_register(rd, (next_program_counter + 4) as u64);
                self.set_delayed_branch_target(branch_target);
            }
            Instruction::Movz(rd, rs, rt) => {
                if self.get_register::<u64>(rt) == 0 {
                    let value = self.get_register::<u64>(rs);
                    self.set_register(rd, value);
                }
            }
            Instruction::Movn(rd, rs, rt) => {
                if self.get_register::<u64>(rt) != 0 {
                    let value = self.get_register::<u64>(rs);
                    self.set_register(rd, value);
                }
            }
            Instruction::Syscall => {
                println!(
                    "v1 register state: {:x}",
                    self.get_register::<u64>(Register::V1)
                );
                println!(
                    "a1 register state: {:x}",
                    self.get_register::<u64>(Register::A1)
                );
                println!("Syscall");
                let syscall_number = self.get_register::<u32>(Register::V1);
                match syscall_number {
                    // SetGsCrt
                    0x02 => {
                        // TODO
                    }
                    // RFU060/initialize main thread
                    0x3c => {
                        let base = self.get_register::<u32>(Register::A1);
                        let size = self.get_register::<u32>(Register::A2);
                        println!("Argument base={:#010x}, size={:#010x}", base, size);
                        let base = if base == 0xFFFF_FFFF {
                            0x0200_0000 - size
                        } else {
                            base
                        };
                        println!("Stack base={:#010x}, size={:#010x}", base, size);
                        let stack_pointer = base + size;
                        self.main_thread_stack_pointer = stack_pointer;
                        self.set_register::<u64>(Register::V0, stack_pointer.sign_extend());
                    }
                    // RFU061/initialize heap
                    0x3d => {
                        let base = self.get_register::<u32>(Register::A0);
                        let size = self.get_register::<u32>(Register::A1);
                        println!("Argument base={:#010x}, size={:#010x}", base, size);
                        let base = if base == 0xFFFF_FFFF {
                            self.main_thread_stack_pointer
                        } else {
                            base
                        };
                        let end = base + size;
                        println!("Heap base={:#010x}, size={:#010x}", base, size);
                        self.set_register::<u64>(Register::V0, end.sign_extend());
                    }
                    // Flush cache
                    0x64 => {}
                    // GsPutIMR
                    0x71 => {}
                    _ => todo!("Syscall number: {syscall_number}"),
                }
            }
            Instruction::Break => todo!(),
            Instruction::Sync => {
                // TODO: maybe do something here
            }
            Instruction::Mfhi(rd) => self.set_register(rd, self.get_register::<u64>(Register::Hi)),
            Instruction::Mthi(_) => todo!(),
            Instruction::Mflo(_) => todo!(),
            Instruction::Mtlo(_) => todo!(),
            Instruction::Dsllv(_, _, _) => todo!(),
            Instruction::Dsrav(_, _, _) => todo!(),
            Instruction::Dsrlv(_, _, _) => todo!(),
            Instruction::Mult(rd, rs, rt) => {
                let a: u64 = self.get_register::<u32>(rs).sign_extend();
                let b: u64 = self.get_register::<u32>(rt).sign_extend();
                let prod = a.wrapping_mul(b);
                let lo: u64 = (prod as u32).sign_extend();
                let hi: u64 = ((prod >> 32) as u32).sign_extend();
                self.set_register(rd, lo);
                self.set_register(Register::Lo, lo);
                self.set_register(Register::Hi, hi);
            }
            Instruction::Multu(rd, rs, rt) => {
                let a = self.get_register::<u32>(rs) as u64;
                let b = self.get_register::<u32>(rt) as u64;
                let prod = a * b;
                let lo: u64 = (prod as u32).sign_extend();
                let hi: u64 = ((prod >> 32) as u32).sign_extend();
                self.set_register(rd, lo);
                self.set_register(Register::Lo, lo);
                self.set_register(Register::Hi, hi);
            }
            Instruction::Div(_, _) => todo!(),
            Instruction::Divu(rs, rt) => {
                let dividend = self.get_register::<u32>(rs);
                let divisor = self.get_register::<u32>(rt);
                let (quotient, remainder) = if divisor == 0 {
                    (!0, dividend)
                } else {
                    (dividend / divisor, dividend % divisor)
                };
                self.set_register::<u64>(Register::Lo, quotient.sign_extend());
                self.set_register::<u64>(Register::Hi, remainder.sign_extend());
            }
            Instruction::Add(rd, rs, rt) => {
                // TODO: Exception on overflow
                let value = self
                    .get_register::<u32>(rs)
                    .wrapping_add(self.get_register::<u32>(rt));
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Addu(rd, rs, rt) => {
                let value = self
                    .get_register::<u32>(rs)
                    .wrapping_add(self.get_register::<u32>(rt));
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Sub(rd, rs, rt) => {
                // TODO: Exception on overflow
                let value = self
                    .get_register::<u32>(rs)
                    .wrapping_sub(self.get_register::<u32>(rt));
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Subu(rd, rs, rt) => {
                self.set_register::<u64>(
                    rd,
                    self.get_register::<u32>(rs)
                        .wrapping_sub(self.get_register::<u32>(rt))
                        .sign_extend(),
                );
            }
            Instruction::And(rd, rs, rt) => {
                self.set_register(
                    rd,
                    self.get_register::<u64>(rs) & self.get_register::<u64>(rt),
                );
            }
            Instruction::Or(rd, rs, rt) => {
                self.set_register(
                    rd,
                    self.get_register::<u64>(rs) | self.get_register::<u64>(rt),
                );
            }
            Instruction::Xor(_, _, _) => todo!(),
            Instruction::Nor(_, _, _) => todo!(),
            Instruction::Mfsa(_) => todo!(),
            Instruction::Mtsa(_) => todo!(),
            Instruction::Slt(rd, rs, rt) => {
                let value = if (self.get_register::<u64>(rs) as i64)
                    < (self.get_register::<u64>(rt) as i64)
                {
                    1
                } else {
                    0
                };
                self.set_register::<u64>(rd, value);
            }
            Instruction::Sltu(rd, rs, rt) => {
                let value = if self.get_register::<u64>(rs) < self.get_register::<u64>(rt) {
                    1
                } else {
                    0
                };
                self.set_register::<u64>(rd, value);
            }
            Instruction::Dadd(_, _, _) => todo!(),
            Instruction::Daddu(rd, rs, rt) => {
                let value = self
                    .get_register::<u64>(rs)
                    .wrapping_add(self.get_register::<u64>(rt));
                self.set_register(rd, value);
            }
            Instruction::Dsub(_, _, _) => todo!(),
            Instruction::Dsubu(_, _, _) => todo!(),
            Instruction::Tge(_, _) => todo!(),
            Instruction::Tgeu(_, _) => todo!(),
            Instruction::Tlt(_, _) => todo!(),
            Instruction::Tltu(_, _) => todo!(),
            Instruction::Teq(_, _) => todo!(),
            Instruction::Tne(_, _) => todo!(),
            Instruction::Dsll(rd, rt, shamt) => {
                self.set_register(rd, self.get_register::<u64>(rt) << shamt);
            }
            Instruction::Dsrl(rd, rt, shamt) => {
                self.set_register(rd, self.get_register::<u64>(rt) >> shamt);
            }
            Instruction::Dsra(_, _, _) => todo!(),
            Instruction::Dsll32(rd, rt, shamt) => {
                self.set_register(rd, self.get_register::<u64>(rt) << (shamt + 32));
            }
            Instruction::Dsrl32(rd, rt, shamt) => {
                self.set_register(rd, self.get_register::<u64>(rt) >> (shamt + 32));
            }
            Instruction::Dsra32(rd, rt, shamt) => {
                self.set_register(
                    rd,
                    ((self.get_register::<u64>(rt) as i64) >> (shamt + 32)) as u64,
                );
            }
            Instruction::Bltz(rs, offset) => {
                if (self.get_register::<u64>(rs) as i64) < 0 {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::Bgez(rs, offset) => {
                if self.get_register::<u64>(rs) as i64 >= 0 {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::J(target) => self.set_delayed_branch_target(
                (next_program_counter & 0xF000_0000).wrapping_add(target << 2),
            ),
            Instruction::Jal(target) => {
                self.set_register(Register::Ra, (next_program_counter + 4) as u64);
                self.set_delayed_branch_target(
                    (next_program_counter & 0xF000_0000).wrapping_add(target << 2),
                );
            }
            Instruction::Beq(rs, rt, offset) => {
                if self.get_register::<u64>(rs) == self.get_register::<u64>(rt) {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::Bne(rs, rt, offset) => {
                if self.get_register::<u64>(rs) != self.get_register::<u64>(rt) {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::Blez(rs, offset) => {
                if (self.get_register::<u64>(rs) as i64) <= 0 {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::Addi(rt, rs, imm) => {
                // TODO exception on overflow
                let temp = self.get_register::<u64>(rs).wrapping_add(imm.sign_extend());
                self.set_register::<u64>(rt, (temp as u32).sign_extend());
            }
            Instruction::Addiu(rt, rs, imm) => {
                let temp = self.get_register::<u64>(rs).wrapping_add(imm.sign_extend());
                self.set_register::<u64>(rt, (temp as u32).sign_extend());
            }
            Instruction::Slti(rt, rs, imm) => {
                let imm: u64 = imm.sign_extend();
                let value = if (self.get_register::<u64>(rs) as i64) < imm as i64 {
                    1
                } else {
                    0
                };
                self.set_register::<u64>(rt, value);
            }
            Instruction::Sltiu(rt, rs, imm) => {
                let imm: u64 = imm.sign_extend();
                let value = if self.get_register::<u64>(rs) < imm {
                    1
                } else {
                    0
                };
                self.set_register::<u64>(rt, value);
            }
            Instruction::Andi(rt, rs, imm) => {
                self.set_register::<u64>(rt, self.get_register::<u64>(rs) & (imm as u64));
            }
            Instruction::Ori(rt, rs, imm) => {
                self.set_register::<u64>(rt, self.get_register::<u64>(rs) | (imm as u64));
            }
            Instruction::Xori(rt, rs, imm) => {
                self.set_register::<u64>(rt, self.get_register::<u64>(rs) ^ (imm as u64));
            }
            Instruction::Lui(rt, imm) => {
                self.set_register::<u64>(rt, ((imm as u32) << 16).sign_extend());
            }
            Instruction::Mfc1(rt, fs) => {
                let value = self.fpu.get_register::<u32>(fs);
                self.set_register::<u64>(rt, value.sign_extend());
            }
            Instruction::Mtc1(rt, fs) => {
                let value = self.get_register::<u32>(rt);
                self.fpu.set_register(fs, value);
            }
            Instruction::Muls(fd, fs, ft) => self.fpu.set_register(
                fd,
                self.fpu.get_register::<f32>(fs) * self.fpu.get_register::<f32>(ft),
            ),
            // TODO flags
            Instruction::Divs(fd, fs, ft) => self.fpu.set_register(
                fd,
                self.fpu.get_register::<f32>(fs) / self.fpu.get_register::<f32>(ft),
            ),
            // TODO flags
            Instruction::Movs(fd, fs) => {
                let value = self.fpu.get_register::<f32>(fs);
                self.fpu.set_register(fd, value);
            }
            Instruction::Cvtws(fd, fs) => {
                let value = self.fpu.get_register::<f32>(fs) as i32;
                self.fpu.set_register(fd, value as u32);
            }
            Instruction::Cvtsw(fd, fs) => {
                let value = self.fpu.get_register::<u32>(fs) as i32;
                self.fpu.set_register(fd, value as f32);
            }
            Instruction::Ei => {
                // TODO: Set status register
            }
            Instruction::Beql(rs, rt, offset) => {
                if self.get_register::<u64>(rs) == self.get_register::<u64>(rt) {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                } else {
                    next_program_counter += 4;
                }
            }
            Instruction::Bnel(rs, rt, offset) => {
                if self.get_register::<u64>(rs) != self.get_register::<u64>(rt) {
                    let offset: u32 = offset.sign_extend();
                    self.set_delayed_branch_target(next_program_counter.wrapping_add(offset << 2));
                } else {
                    next_program_counter += 4;
                }
            }
            Instruction::Mult1(rd, rs, rt) => {
                let a: u64 = self.get_register::<u32>(rs).sign_extend();
                let b: u64 = self.get_register::<u32>(rt).sign_extend();
                let prod = a.wrapping_mul(b);
                let lo: u64 = (prod as u32).sign_extend();
                let hi: u64 = ((prod >> 32) as u32).sign_extend();
                self.set_register(rd, lo);
                self.set_register::<u128>(
                    Register::Lo,
                    ((lo as u128) << 64) | self.get_register::<u64>(Register::Lo) as u128,
                );
                self.set_register::<u128>(
                    Register::Hi,
                    ((hi as u128) << 64) | self.get_register::<u64>(Register::Hi) as u128,
                );
            }
            Instruction::Sq(rt, base, offset) => {
                let mut address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                address &= !0b1111;
                self.write_virtual(bus, address, self.get_register::<u128>(rt));
            }
            Instruction::Lb(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                let value = self.read_virtual::<u8>(bus, address);
                self.set_register::<u64>(rt, value.sign_extend());
            }
            Instruction::Lh(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..1) != 0 {
                    panic!("Unaligned load at {:#010x}", address);
                }
                let value = self.read_virtual::<u16>(bus, address);
                self.set_register::<u64>(rt, value.sign_extend());
            }
            Instruction::Lw(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..2) != 0 {
                    panic!("Unaligned load at {:#010x}", address);
                }
                let value = self.read_virtual::<u32>(bus, address);
                self.set_register::<u64>(rt, value.sign_extend());
            }
            Instruction::Lbu(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                let value = self.read_virtual::<u8>(bus, address);
                self.set_register(rt, value as u64);
            }
            Instruction::Lhu(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                let value = self.read_virtual::<u16>(bus, address);
                self.set_register(rt, value as u64);
            }
            Instruction::Lwr(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                let byte = address & 0b11;
                let memory_word = self.read_virtual::<u32>(bus, address & !0b11);
                let value = if byte == 0 {
                    memory_word.sign_extend()
                } else {
                    let existing = self.get_register::<u64>(rt);
                    existing & u64::mask(byte * 8..64) | (memory_word >> (byte * 8)) as u64
                };
                self.set_register(rt, value);
            }
            Instruction::Sb(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                self.write_virtual(bus, address, self.get_register::<u8>(rt));
            }
            Instruction::Sh(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..1) != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                self.write_virtual(bus, address, self.get_register::<u16>(rt));
            }
            Instruction::Sw(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..2) != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                self.write_virtual(bus, address, self.get_register::<u32>(rt));
            }
            Instruction::Lwc1(ft, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..2) != 0 {
                    panic!("Unaligned load at {:#010x}", address);
                }
                let value = self.read_virtual::<u32>(bus, address);
                self.fpu.set_register(ft, value);
            }
            Instruction::Ld(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..3) != 0 {
                    panic!("Unaligned load at {:#010x}", address);
                }
                let value = self.read_virtual(bus, address);
                self.set_register::<u64>(rt, value);
            }
            Instruction::Swc1(ft, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..2) != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                self.write_virtual(bus, address, self.fpu.get_register::<u32>(ft));
            }
            Instruction::Sd(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address.bits(0..3) != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                self.write_virtual(bus, address, self.get_register::<u64>(rt));
            }
        }
        // for reg in instruction.definitions() {
        //     match reg {
        //         AnyRegister::Core(reg) => {
        //             let value = self.get_register::<u64>(reg);
        //             println!("{}:={:#x}", reg, value);
        //         }
        //         AnyRegister::Fpu(_) => {}
        //     }
        // }
        self.program_counter = next_program_counter;
    }
}
