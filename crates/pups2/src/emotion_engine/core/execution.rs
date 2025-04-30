use crate::emotion_engine::bus::Bus;

use super::{instruction_gen::Instruction, jit::Code, Core};

impl Core {
    pub fn step(&mut self, mut cycles: u64, bus: &mut Bus) {
        self.state.control.step(cycles);
        while cycles > 0 {
            if self.state.delayed_branch_target.is_some() {
                cycles -= 1;
                let instruction =
                    Instruction::decode(self.read_virtual(bus, self.state.program_counter));
                // println!("I {:08x}: {}", self.state.program_counter, instruction);
                self.interpret_instruction(instruction, bus);
            } else {
                let cache_entry = self.jit.cache_entry(&self.state, &self.mmu, bus, self.mode);
                // TODO: Better cycle counting
                match &cache_entry.code {
                    Code::Jitted(function) => {
                        let bytes = cache_entry.address_range.end - cache_entry.address_range.start;
                        // for pc in (self.state.program_counter..self.state.program_counter + bytes)
                        //     .step_by(4)
                        // {
                        //     let physical_address = self.mmu.virtual_to_physical(pc, self.mode);
                        //     let instruction = decode(bus.read(physical_address));
                        //     println!("J {:08x}: {}", pc, instruction);
                        // }
                        assert!(self.mmu.physically_consecutive(
                            self.state.program_counter..self.state.program_counter + bytes,
                            self.mode
                        ));
                        cycles = cycles.saturating_sub(bytes as u64 / 4);
                        function(self.mode);
                    }
                    Code::Interpreted(instruction) => {
                        cycles -= 1;
                        let instruction = *instruction;
                        // println!("I {:08x}: {}", self.state.program_counter, instruction);
                        self.interpret_instruction(instruction, bus);
                    }
                }
            }
        }
    }
}
