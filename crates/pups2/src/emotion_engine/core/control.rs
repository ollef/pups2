use derive_more::Display;
use enum_map::{enum_map, Enum, EnumMap};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::bits::Bits;

// Coprocessor 0
#[derive(Debug)]
pub struct Control {
    registers: EnumMap<Register, u32>,
}

impl Control {
    pub fn new() -> Control {
        Control {
            registers: enum_map! {
                Register::PrId => 0x2E20,
                Register::Config => 0x440,
                _ => 0,
            },
        }
    }

    pub fn step(&mut self, cycles: u64) {
        self.registers[Register::Count] =
            self.registers[Register::Count].wrapping_add(cycles as u32);
    }

    pub fn get_register(&self, register: Register) -> u32 {
        match register {
            Register::Index => self.registers[register],
            Register::Random => todo!(),
            Register::EntryLo0 => self.registers[register],
            Register::EntryLo1 => self.registers[register],
            Register::Context => todo!(),
            Register::PageMask => self.registers[register],
            Register::Wired => todo!(),
            Register::Undefined7 => todo!(),
            Register::BadVAddr => todo!(),
            Register::Count => self.registers[register],
            Register::EntryHi => self.registers[register],
            Register::Compare => todo!(),
            Register::Status => todo!(),
            Register::Cause => todo!(),
            Register::Epc => todo!(),
            Register::PrId => self.registers[register],
            Register::Config => todo!(),
            Register::Undefined17 => todo!(),
            Register::Undefined18 => todo!(),
            Register::Undefined19 => todo!(),
            Register::Undefined20 => todo!(),
            Register::Undefined21 => todo!(),
            Register::Undefined22 => todo!(),
            Register::BadPAddr => todo!(),
            Register::Undefined24 => todo!(),
            Register::Undefined25 => todo!(),
            Register::Undefined26 => todo!(),
            Register::Undefined27 => todo!(),
            Register::TagLo => todo!(),
            Register::TagHi => todo!(),
            Register::ErrorEpc => todo!(),
            Register::Undefined31 => todo!(),
        }
    }

    pub fn set_register(&mut self, register: Register, value: u32) {
        println!("Setting control register {:?} to {:#010x}", register, value);
        let register_value = &mut self.registers[register];
        match register {
            Register::Index => {
                *register_value = value;
                register_value.set_bits(6..=30, 0u32);
            }
            Register::Random => todo!(),
            Register::EntryLo0 => {
                *register_value = value;
                register_value.set_bits(26..=30, 0u32);
            }
            Register::EntryLo1 => {
                *register_value = value;
                register_value.set_bits(26..=31, 0u32);
            }
            Register::Context => todo!(),
            Register::PageMask => register_value.set_bits(13..=24, value.bits(13..=24)),
            Register::Wired => register_value.set_bits(0..=5, value.bits(0..=5)),
            Register::Undefined7 => todo!(),
            Register::BadVAddr => todo!(),
            Register::Count => *register_value = value,
            Register::EntryHi => {
                *register_value = value;
                register_value.set_bits(8..=12, 0u32);
            }
            Register::Compare => *register_value = value,
            Register::Status => {
                *register_value = value;
                register_value.set_bits(5..10, 0u32);
                register_value.set_bits(19..22, 0u32);
                register_value.set_bits(24..28, 0u32);
            }
            Register::Cause => todo!(),
            Register::Epc => todo!(),
            Register::PrId => todo!(),
            Register::Config => {
                *register_value = value;
                register_value.set_bits(3..=5, 0u32);
                register_value.set_bits(14..=15, 0u32);
                register_value.set_bits(19..=27, 0u32);
                register_value.set_bit(31, false);
            }
            Register::Undefined17 => todo!(),
            Register::Undefined18 => todo!(),
            Register::Undefined19 => todo!(),
            Register::Undefined20 => todo!(),
            Register::Undefined21 => todo!(),
            Register::Undefined22 => todo!(),
            Register::BadPAddr => todo!(),
            Register::Undefined24 => todo!(),
            Register::Undefined25 => todo!(),
            Register::Undefined26 => todo!(),
            Register::Undefined27 => todo!(),
            Register::TagLo => todo!(),
            Register::TagHi => todo!(),
            Register::ErrorEpc => todo!(),
            Register::Undefined31 => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum, Display, FromPrimitive)]
pub enum Register {
    Index,
    Random,
    EntryLo0,
    EntryLo1,
    Context,
    PageMask,
    Wired,
    Undefined7,
    BadVAddr,
    Count,
    EntryHi,
    Compare,
    Status,
    Cause,
    Epc,
    PrId,
    Config,
    Undefined17,
    Undefined18,
    Undefined19,
    Undefined20,
    Undefined21,
    Undefined22,
    BadPAddr,
    Undefined24,
    Undefined25,
    Undefined26,
    Undefined27,
    TagLo,
    TagHi,
    ErrorEpc,
    Undefined31,
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        Register::from_u32(value & 0b11111).unwrap()
    }
}
