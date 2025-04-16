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
        self.registers[register]
    }

    pub fn set_register(&mut self, register: Register, value: u32) {
        self.registers[register] = value;
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
