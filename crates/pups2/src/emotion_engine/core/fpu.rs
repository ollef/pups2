use std::fmt::Display;

use super::register::{GetRegister, SetRegister};

// Coprocessor 1
#[derive(Debug)]
pub struct Fpu {
    registers: [f32; 32],
}

impl Fpu {
    pub fn new() -> Fpu {
        Fpu {
            registers: [0.0; 32],
        }
    }

    pub fn get_register<T>(&self, register: Register) -> T
    where
        f32: GetRegister<T>,
    {
        self.registers[register.index as usize].get_register()
    }

    pub fn set_register<T>(&mut self, register: Register, value: T)
    where
        f32: SetRegister<T>,
    {
        self.registers[register.index as usize].set_register(value);
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Register {
    index: u8,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.index)
    }
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        Register {
            index: value as u8 & 0b11111,
        }
    }
}

impl GetRegister<f32> for f32 {
    fn get_register(&self) -> f32 {
        *self
    }
}

impl GetRegister<u32> for f32 {
    fn get_register(&self) -> u32 {
        self.to_bits()
    }
}

impl SetRegister<f32> for f32 {
    fn set_register(&mut self, value: f32) {
        *self = value;
    }
}

impl SetRegister<u32> for f32 {
    fn set_register(&mut self, value: u32) {
        *self = f32::from_bits(value);
    }
}
