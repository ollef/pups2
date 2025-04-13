use crate::emotion_engine::bus::Bus;

use super::{jit::Code, Core};

impl Core {
    pub fn step(&mut self, mut cycles: u64, bus: &mut Bus) {
        while cycles > 0 {
            let cache_entry = self
                .jit
                .cache_entry(self.program_counter, &self.mmu, bus, self.mode);
            // TODO: Better cycle counting
            cycles = cycles.saturating_sub(cache_entry.address_range.len() as u64 / 4);
            match &cache_entry.code {
                Code::Jitted(function) => function(),
                Code::Interpreted(instruction) => {
                    let instruction = *instruction;
                    self.interpret_instruction(instruction, bus)
                }
            }
        }
    }
}
