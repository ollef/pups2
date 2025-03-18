use crate::emotion_engine::core::register::Register;

use super::{disassembler::disassemble, instruction::Instruction, state::State};

trait SignExtend<T> {
    fn sign_extend(self) -> T;
}

impl<T> SignExtend<T> for u16
where
    i16: SignExtend<T>,
{
    fn sign_extend(self) -> T {
        (self as i16).sign_extend()
    }
}

impl<T> SignExtend<T> for u32
where
    i32: SignExtend<T>,
{
    fn sign_extend(self) -> T {
        (self as i32).sign_extend()
    }
}

impl SignExtend<u32> for i16 {
    fn sign_extend(self) -> u32 {
        self as i32 as u32
    }
}

impl SignExtend<u64> for i16 {
    fn sign_extend(self) -> u64 {
        self as i64 as u64
    }
}

impl SignExtend<u64> for i32 {
    fn sign_extend(self) -> u64 {
        self as i64 as u64
    }
}

impl State {
    pub fn set_delayed_branch_target(&mut self, target: u32) {
        if target & 0b11 != 0 {
            panic!("Invalid branch target: {:#010x}", target);
        }
        self.delayed_branch_target = Some(target);
    }

    pub fn step_interpreter(&mut self) {
        let physical_program_counter = self
            .mmu
            .virtual_to_physical(self.program_counter, self.mode);
        let raw_instruction = self
            .bus
            .read(physical_program_counter)
            .expect("Failed to read instruction");
        let instruction = disassemble(raw_instruction);
        for reg in instruction.uses() {
            let value = self.get_register::<u64>(reg);
            println!("{}={:#x}", reg, value);
        }
        let delayed_branch_target = self.delayed_branch_target.take();
        println!("pc={:#010}: {instruction}", self.program_counter);
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
                let value =
                    self.get_register::<u32>(rt) << (self.get_register::<u32>(rs) & 0b11111);
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Srlv(rd, rt, rs) => {
                let value =
                    self.get_register::<u32>(rt) >> (self.get_register::<u32>(rs) & 0b11111);
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Srav(rd, rt, rs) => {
                let value = (self.get_register::<u32>(rt) as i32)
                    >> (self.get_register::<u32>(rs) & 0b11111);
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Jr(rs) => {
                self.set_delayed_branch_target(self.get_register::<u32>(rs));
            }
            Instruction::Jalr(rd, rs) => {
                self.set_register(rd, self.program_counter as u64 + 8);
                let branch_target = self.get_register::<u32>(rs);
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
                        let stack_address = if base == 0xFFFF_FFFF {
                            0x0200_0000
                        } else {
                            base + size
                        } - 0x2A0;
                        self.main_thread_stack_base = stack_address;
                        self.set_register::<u64>(Register::V0, stack_address.sign_extend());
                    }
                    // RFU061/initialize heap
                    0x3d => {
                        let base = self.get_register::<u32>(Register::A0);
                        let size = self.get_register::<u32>(Register::A1);
                        let heap_address = if size == 0xFFFF_FFFF {
                            self.main_thread_stack_base
                        } else {
                            base + size
                        };
                        self.set_register::<u64>(Register::V0, heap_address.sign_extend());
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
            Instruction::Mfhi(_) => todo!(),
            Instruction::Mthi(_) => todo!(),
            Instruction::Mflo(_) => todo!(),
            Instruction::Mtlo(_) => todo!(),
            Instruction::Dsllv(_, _, _) => todo!(),
            Instruction::Dsrav(_, _, _) => todo!(),
            Instruction::Dsrlv(_, _, _) => todo!(),
            Instruction::Mult(rd, rs, rt) => {
                let a: u64 = self.get_register::<u32>(rs).sign_extend();
                let b: u64 = self.get_register::<u32>(rt).sign_extend();
                let prod = a * b;
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
            Instruction::Divu(_, _) => todo!(),
            Instruction::Add(_, _, _) => todo!(),
            Instruction::Addu(rd, rs, rt) => {
                let value = self
                    .get_register::<u32>(rs)
                    .wrapping_add(self.get_register::<u32>(rt));
                self.set_register::<u64>(rd, value.sign_extend());
            }
            Instruction::Sub(_, _, _) => todo!(),
            Instruction::Subu(_, _, _) => todo!(),
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
            Instruction::Slt(_, _, _) => todo!(),
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
            Instruction::Dsrl32(_, _, _) => todo!(),
            Instruction::Dsra32(_, _, _) => todo!(),
            Instruction::Bgez(_, _) => todo!(),
            Instruction::J(_) => todo!(),
            Instruction::Jal(target) => {
                self.set_register(Register::Ra, (self.program_counter + 8) as u64);
                self.set_delayed_branch_target(
                    ((self.program_counter + 4) & 0xF000_0000) + (target << 2),
                );
            }
            Instruction::Beq(rs, rt, offset) => {
                let offset: u32 = offset.sign_extend();
                if self.get_register::<u64>(rs) == self.get_register::<u64>(rt) {
                    self.set_delayed_branch_target(self.program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::Bne(rs, rt, offset) => {
                let offset: u32 = offset.sign_extend();
                if self.get_register::<u64>(rs) != self.get_register::<u64>(rt) {
                    self.set_delayed_branch_target(self.program_counter.wrapping_add(offset << 2));
                }
            }
            Instruction::Addiu(rt, rs, imm) => {
                let temp = self.get_register::<u64>(rs).wrapping_add(imm.sign_extend());
                self.set_register::<u64>(rt, (temp as u32).sign_extend());
            }
            Instruction::Andi(rt, rs, imm) => {
                self.set_register::<u64>(rt, self.get_register::<u64>(rs) & (imm as u64));
            }
            Instruction::Ori(rt, rs, imm) => {
                self.set_register::<u64>(rt, self.get_register::<u64>(rs) | (imm as u64));
            }
            Instruction::Lui(rt, imm) => {
                self.set_register::<u64>(rt, ((imm as u32) << 16).sign_extend());
            }
            Instruction::Ei => {
                // TODO: Set status register
            }
            Instruction::Sq(rt, base, offset) => {
                let mut address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                address &= !0b1111;
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                self.bus
                    .write(physical_address, self.get_register::<u128>(rt));
            }
            Instruction::Lh(_, _, _) => todo!(),
            Instruction::Lw(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address & 0b11 != 0 {
                    panic!("Unaligned load at {:#010x}", address);
                }
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                let value = self
                    .bus
                    .read::<u32>(physical_address)
                    .expect("Failed to read word");
                self.set_register::<u64>(rt, value.sign_extend());
            }
            Instruction::Lbu(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                let value = self
                    .bus
                    .read::<u8>(physical_address)
                    .expect("Failed to read byte");
                self.set_register(rt, value as u64);
            }
            Instruction::Lwr(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                let byte = address & 0b11;
                let memory_word = self
                    .bus
                    .read::<u32>(physical_address & !0b11)
                    .expect("Failed to read word");
                let value = if byte == 0 {
                    memory_word.sign_extend()
                } else {
                    let existing = self.get_register::<u64>(rt);
                    (existing & (!0 << ((4 - byte) * 8))) | (memory_word >> (byte * 8)) as u64
                };
                self.set_register(rt, value);
            }
            Instruction::Sb(_, _, _) => todo!(),
            Instruction::Sh(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address & 0b1 != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                self.bus
                    .write(physical_address, self.get_register::<u16>(rt));
            }
            Instruction::Sw(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address & 0b11 != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                self.bus
                    .write(physical_address, self.get_register::<u32>(rt));
            }
            Instruction::Ld(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address & 0b111 != 0 {
                    panic!("Unaligned load at {:#010x}", address);
                }
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                let value = self
                    .bus
                    .read(physical_address)
                    .expect("Failed to read double word");
                self.set_register::<u64>(rt, value);
            }
            Instruction::Sd(rt, base, offset) => {
                let address = self
                    .get_register::<u32>(base)
                    .wrapping_add(offset.sign_extend());
                if address & 0b111 != 0 {
                    panic!("Unaligned store at {:#010x}", address);
                }
                let physical_address = self.mmu.virtual_to_physical(address, self.mode);
                self.bus
                    .write(physical_address, self.get_register::<u64>(rt));
            }
        }
        for reg in instruction.definitions() {
            let value = self.get_register::<u64>(reg);
            println!("{}:={:#x}", reg, value);
        }
        if let Some(branch_target) = delayed_branch_target {
            self.program_counter = branch_target;
        } else {
            self.program_counter += 4;
        }
    }
}
