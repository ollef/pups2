use std::collections::VecDeque;

use privileged_registers::PrivilegedRegisters;
pub use registers::{Register, Registers};
use rendering::Vertex;

use crate::fifo::Fifo;

mod pixel_storage;
mod privileged_registers;
mod registers;
mod rendering;

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pub command_queue: VecDeque<(Register, u64)>,
    privileged_registers: PrivilegedRegisters,
    registers: Registers,
    vertex_queue: Fifo<Vertex>,
    tmp_data: Vec<u8>,
}

impl Gs {
    pub fn new() -> Gs {
        Gs {
            local_memory: vec![0; LOCAL_MEMORY_SIZE].into_boxed_slice(),
            command_queue: VecDeque::new(),
            privileged_registers: PrivilegedRegisters::default(),
            registers: Registers::default(),
            vertex_queue: Fifo::with_capacity(2),
            tmp_data: Vec::new(),
        }
    }

    pub fn step(&mut self) {
        while let Some((register, data)) = self.command_queue.pop_front() {
            // println!("Command: {:?}={:x?}", register, data);
            self.write_register(register, data);
        }
    }
}
