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
    pub fn interpret(&mut self, instruction: &Instruction) {
        match instruction {
            _ => panic!("Unimplemented instruction: {}", instruction),
        }
    }
}
