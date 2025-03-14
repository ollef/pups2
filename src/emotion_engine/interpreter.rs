use super::{instruction::Instruction, state::State};

impl State {
    pub fn interpret(&mut self, instruction: &Instruction) {
        match instruction {
            _ => panic!("Unimplemented instruction: {}", instruction),
        }
    }
}
