use super::{instruction::Instruction, state::State};

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

    pub fn interpret(&mut self, instruction: &Instruction) {
        let delayed_branch_target = self.delayed_branch_target.take();
        match instruction {
            Instruction::Unknown => {
                println!("Unknown instruction at {:#010x}", self.program_counter)
            }
            Instruction::Sll(rd, rt, shamt) => {
                let value = self.registers[*rt].read32() << shamt;
                self.registers[*rd].write64(value.sign_extend());
            }
            Instruction::Srl(rd, rt, shamt) => {
                let value = self.registers[*rt].read32() >> shamt;
                self.registers[*rd].write64(value.sign_extend());
            }
            Instruction::Sra(rd, rt, shamt) => {
                let value = (self.registers[*rt].read32() as i32) >> shamt;
                self.registers[*rd].write64(value.sign_extend());
            }
            Instruction::Sllv(rd, rt, rs) => {
                let value =
                    self.registers[*rt].read32() << (self.registers[*rs].read32() & 0b11111);
                self.registers[*rd].write64(value.sign_extend());
            }
            Instruction::Srlv(rd, rt, rs) => {
                let value =
                    self.registers[*rt].read32() >> (self.registers[*rs].read32() & 0b11111);
                self.registers[*rd].write64(value.sign_extend());
            }
            Instruction::Srav(rd, rt, rs) => {
                let value = (self.registers[*rt].read32() as i32)
                    >> (self.registers[*rs].read32() & 0b11111);
                self.registers[*rd].write64(value.sign_extend());
            }
            Instruction::Jr(rs) => {
                self.set_delayed_branch_target(self.registers[*rs].read32());
            }
            Instruction::Jalr(rd, rs) => {
                self.registers[*rd].write64(self.program_counter as u64 + 8);
                let branch_target = self.registers[*rs].read32();
                self.set_delayed_branch_target(branch_target);
            }
            Instruction::Movz(rd, rs, rt) => {
                if self.registers[*rt].read64() == 0 {
                    let value = self.registers[*rs].read64();
                    self.registers[*rd].write64(value);
                }
            }
            Instruction::Movn(rd, rs, rt) => {
                if self.registers[*rt].read64() != 0 {
                    let value = self.registers[*rs].read64();
                    self.registers[*rd].write64(value);
                }
            }
            _ => panic!("Unimplemented instruction: {}", instruction),
        }
        if let Some(branch_target) = delayed_branch_target {
            self.program_counter = branch_target;
        } else {
            self.program_counter += 4;
        }
    }
}
