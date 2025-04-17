use derive_more::Display;
use enum_map::{enum_map, Enum, EnumMap};

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
            Register::Count => todo!(),
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
        match register {
            Register::Index => self.registers[register] = value,
            Register::Random => todo!(),
            Register::EntryLo0 => self.registers[register] = value,
            Register::EntryLo1 => self.registers[register] = value,
            Register::Context => todo!(),
            Register::PageMask => self.registers[register] = value,
            Register::Wired => todo!(),
            Register::Undefined7 => todo!(),
            Register::BadVAddr => todo!(),
            Register::Count => self.registers[register] = value,
            Register::EntryHi => self.registers[register] = value,
            Register::Compare => self.registers[register] = value,
            Register::Status => self.registers[register] = value,
            Register::Cause => todo!(),
            Register::Epc => todo!(),
            Register::PrId => todo!(),
            Register::Config => self.registers[register] = value,
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum, Display)]
#[repr(u8)]
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
        Register::from_usize((value & 0b11111) as usize)
    }
}
