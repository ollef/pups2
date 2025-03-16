use std::fmt::Display;

use enum_map::Enum;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Register {
    Zero,
    At,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    Gp,
    Sp,
    Fp,
    Ra,
    Lo,
    Hi,
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        let value = value & 0b11111;
        unsafe { std::mem::transmute(value as u8) }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Zero => write!(f, "zero"),
            Register::At => write!(f, "at"),
            Register::V0 => write!(f, "v0"),
            Register::V1 => write!(f, "v1"),
            Register::A0 => write!(f, "a0"),
            Register::A1 => write!(f, "a1"),
            Register::A2 => write!(f, "a2"),
            Register::A3 => write!(f, "a3"),
            Register::T0 => write!(f, "t0"),
            Register::T1 => write!(f, "t1"),
            Register::T2 => write!(f, "t2"),
            Register::T3 => write!(f, "t3"),
            Register::T4 => write!(f, "t4"),
            Register::T5 => write!(f, "t5"),
            Register::T6 => write!(f, "t6"),
            Register::T7 => write!(f, "t7"),
            Register::S0 => write!(f, "s0"),
            Register::S1 => write!(f, "s1"),
            Register::S2 => write!(f, "s2"),
            Register::S3 => write!(f, "s3"),
            Register::S4 => write!(f, "s4"),
            Register::S5 => write!(f, "s5"),
            Register::S6 => write!(f, "s6"),
            Register::S7 => write!(f, "s7"),
            Register::T8 => write!(f, "t8"),
            Register::T9 => write!(f, "t9"),
            Register::K0 => write!(f, "k0"),
            Register::K1 => write!(f, "k1"),
            Register::Gp => write!(f, "gp"),
            Register::Sp => write!(f, "sp"),
            Register::Fp => write!(f, "fp"),
            Register::Ra => write!(f, "ra"),
            Register::Lo => write!(f, "lo"),
            Register::Hi => write!(f, "hi"),
        }
    }
}

impl Register {
    pub fn non_zero(self) -> Option<Self> {
        match self {
            Register::Zero => None,
            _ => Some(self),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum)]
pub enum Cop0Register {
    Index = 0,
    Random = 1,
    EntryLo0 = 2,
    EntryLo1 = 3,
    Context = 4,
    PageMask = 5,
    Wired = 6,
    BadVAddr = 8,
    Count = 9,
    EntryHi = 10,
    Compare = 11,
    Status = 12,
    Cause = 13,
    Epc = 14,
    PrId = 15,
    Config = 16,
    BadPAddr = 23,
    TagLo = 28,
    TagHi = 29,
    ErrorEpc = 30,
    Undefined = 31,
}

impl From<u32> for Cop0Register {
    fn from(value: u32) -> Self {
        match value & 0b11111 {
            0 => Cop0Register::Index,
            1 => Cop0Register::Random,
            2 => Cop0Register::EntryLo0,
            3 => Cop0Register::EntryLo1,
            4 => Cop0Register::Context,
            5 => Cop0Register::PageMask,
            6 => Cop0Register::Wired,
            8 => Cop0Register::BadVAddr,
            9 => Cop0Register::Count,
            10 => Cop0Register::EntryHi,
            11 => Cop0Register::Compare,
            12 => Cop0Register::Status,
            13 => Cop0Register::Cause,
            14 => Cop0Register::Epc,
            15 => Cop0Register::PrId,
            16 => Cop0Register::Config,
            23 => Cop0Register::BadPAddr,
            28 => Cop0Register::TagLo,
            29 => Cop0Register::TagHi,
            30 => Cop0Register::ErrorEpc,
            _ => Cop0Register::Undefined,
        }
    }
}
