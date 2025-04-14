use crate::emotion_engine::{bus::Bus, core::decoder::decode};

use super::{jit::Code, Core};

impl Core {
    pub fn step(&mut self, mut cycles: u64, bus: &mut Bus) {
        while cycles > 0 {
            let cache_entry = self.jit.cache_entry(&self.state, &self.mmu, bus, self.mode);
            // TODO: Better cycle counting
            match &cache_entry.code {
                Code::Jitted(_) if self.state.delayed_branch_target.is_some() => {
                    cycles -= 1;
                    let instruction = decode(self.read_virtual(bus, self.state.program_counter));
                    self.interpret_instruction(instruction, bus);
                }
                Code::Jitted(function) => {
                    let bytes = cache_entry.address_range.end - cache_entry.address_range.start;
                    assert!(self.mmu.physically_consecutive(
                        self.state.program_counter..self.state.program_counter + bytes,
                        self.mode
                    ));
                    cycles = cycles.saturating_sub(bytes as u64 / 4);
                    function();
                }
                Code::Interpreted(instruction) => {
                    cycles -= 1;
                    let instruction = *instruction;
                    self.interpret_instruction(instruction, bus);
                }
            }
        }
    }
}
