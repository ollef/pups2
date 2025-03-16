use super::{disassembler::disassemble, instruction::Instruction, state::State};

trait SignExtend<T> {
    fn sign_extend(self) -> T;
}

impl SignExtend<u64> for u32 {
    fn sign_extend(self) -> u64 {
        (self as i32).sign_extend()
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
            .tlb
            .virtual_to_physical(self.program_counter, self.mode);
        let raw_instruction = self
            .memory
            .read32(physical_program_counter)
            .expect("Failed to read instruction");
        let instruction = disassemble(raw_instruction);
        let delayed_branch_target = self.delayed_branch_target.take();
        println!("Interpreting {instruction}");
        match instruction {
            Instruction::Unknown => {
                println!("Unknown instruction at {:#010x}", self.program_counter)
            }
            Instruction::Sll(rd, rt, shamt) => {
                let value = self.registers[rt].read32() << shamt;
                self.registers[rd].write64(value.sign_extend());
            }
            Instruction::Srl(rd, rt, shamt) => {
                let value = self.registers[rt].read32() >> shamt;
                self.registers[rd].write64(value.sign_extend());
            }
            Instruction::Sra(rd, rt, shamt) => {
                let value = (self.registers[rt].read32() as i32) >> shamt;
                self.registers[rd].write64(value.sign_extend());
            }
            Instruction::Sllv(rd, rt, rs) => {
                let value = self.registers[rt].read32() << (self.registers[rs].read32() & 0b11111);
                self.registers[rd].write64(value.sign_extend());
            }
            Instruction::Srlv(rd, rt, rs) => {
                let value = self.registers[rt].read32() >> (self.registers[rs].read32() & 0b11111);
                self.registers[rd].write64(value.sign_extend());
            }
            Instruction::Srav(rd, rt, rs) => {
                let value =
                    (self.registers[rt].read32() as i32) >> (self.registers[rs].read32() & 0b11111);
                self.registers[rd].write64(value.sign_extend());
            }
            Instruction::Jr(rs) => {
                self.set_delayed_branch_target(self.registers[rs].read32());
            }
            Instruction::Jalr(rd, rs) => {
                self.registers[rd].write64(self.program_counter as u64 + 8);
                let branch_target = self.registers[rs].read32();
                self.set_delayed_branch_target(branch_target);
            }
            Instruction::Movz(rd, rs, rt) => {
                if self.registers[rt].read64() == 0 {
                    let value = self.registers[rs].read64();
                    self.registers[rd].write64(value);
                }
            }
            Instruction::Movn(rd, rs, rt) => {
                if self.registers[rt].read64() != 0 {
                    let value = self.registers[rs].read64();
                    self.registers[rd].write64(value);
                }
            }
            Instruction::Syscall => todo!(),
            Instruction::Break => todo!(),
            Instruction::Sync => todo!(),
            Instruction::Mfhi(_) => todo!(),
            Instruction::Mthi(_) => todo!(),
            Instruction::Mflo(_) => todo!(),
            Instruction::Mtlo(_) => todo!(),
            Instruction::Dsllv(_, _, _) => todo!(),
            Instruction::Dsrav(_, _, _) => todo!(),
            Instruction::Dsrlv(_, _, _) => todo!(),
            Instruction::Mult(_, _) => todo!(),
            Instruction::Multu(_, _) => todo!(),
            Instruction::Div(_, _) => todo!(),
            Instruction::Divu(_, _) => todo!(),
            Instruction::Add(_, _, _) => todo!(),
            Instruction::Addu(_, _, _) => todo!(),
            Instruction::Sub(_, _, _) => todo!(),
            Instruction::Subu(_, _, _) => todo!(),
            Instruction::And(_, _, _) => todo!(),
            Instruction::Or(_, _, _) => todo!(),
            Instruction::Xor(_, _, _) => todo!(),
            Instruction::Nor(_, _, _) => todo!(),
            Instruction::Mfsa(_) => todo!(),
            Instruction::Mtsa(_) => todo!(),
            Instruction::Slt(_, _, _) => todo!(),
            Instruction::Sltu(_, _, _) => todo!(),
            Instruction::Dadd(_, _, _) => todo!(),
            Instruction::Daddu(_, _, _) => todo!(),
            Instruction::Dsub(_, _, _) => todo!(),
            Instruction::Dsubu(_, _, _) => todo!(),
            Instruction::Tge(_, _) => todo!(),
            Instruction::Tgeu(_, _) => todo!(),
            Instruction::Tlt(_, _) => todo!(),
            Instruction::Tltu(_, _) => todo!(),
            Instruction::Teq(_, _) => todo!(),
            Instruction::Tne(_, _) => todo!(),
            Instruction::Dsll(_, _, _) => todo!(),
            Instruction::Dsrl(_, _, _) => todo!(),
            Instruction::Dsra(_, _, _) => todo!(),
            Instruction::Dsll32(_, _, _) => todo!(),
            Instruction::Dsrl32(_, _, _) => todo!(),
            Instruction::Dsra32(_, _, _) => todo!(),
            Instruction::Bgez(_, _) => todo!(),
            Instruction::J(_) => todo!(),
            Instruction::Jal(_) => todo!(),
            Instruction::Beq(_, _, _) => todo!(),
            Instruction::Bne(_, _, _) => todo!(),
            Instruction::Addiu(_, _, _) => todo!(),
            Instruction::Andi(_, _, _) => todo!(),
            Instruction::Ori(_, _, _) => todo!(),
            Instruction::Lui(_, _) => todo!(),
            Instruction::Ei => todo!(),
            Instruction::Sq(_, _, _) => todo!(),
            Instruction::Lh(_, _, _) => todo!(),
            Instruction::Lw(_, _, _) => todo!(),
            Instruction::Lbu(_, _, _) => todo!(),
            Instruction::Lwr(_, _, _) => todo!(),
            Instruction::Sb(_, _, _) => todo!(),
            Instruction::Sh(_, _, _) => todo!(),
            Instruction::Sw(_, _, _) => todo!(),
            Instruction::Ld(_, _, _) => todo!(),
            Instruction::Sd(_, _, _) => todo!(),
        }
        if let Some(branch_target) = delayed_branch_target {
            self.program_counter = branch_target;
        } else {
            self.program_counter += 4;
        }
    }
}
