use std::fmt::Display;

use enum_map::Enum;

use super::fpu;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum)]
#[repr(u8)]
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
#[repr(u8)]
pub enum ControlRegister {
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

impl From<u32> for ControlRegister {
    fn from(value: u32) -> Self {
        let value = value & 0b11111;
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AnyRegister {
    Core(Register),
    Fpu(fpu::Register),
}

impl AnyRegister {
    pub fn non_zero(self) -> Option<Self> {
        match self {
            AnyRegister::Core(register) => register.non_zero().map(AnyRegister::Core),
            AnyRegister::Fpu(register) => Some(AnyRegister::Fpu(register)),
        }
    }
}

pub trait SetRegister<T> {
    fn set_register(&mut self, value: T);
}

pub trait GetRegister<T> {
    fn get_register(&self) -> T;
}

impl SetRegister<u64> for u128 {
    #[inline(always)]
    fn set_register(&mut self, value: u64) {
        *self = value as u128 | (*self & 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000);
    }
}

impl SetRegister<u128> for u128 {
    #[inline(always)]
    fn set_register(&mut self, value: u128) {
        *self = value;
    }
}

impl GetRegister<u8> for u128 {
    #[inline(always)]
    fn get_register(&self) -> u8 {
        *self as u8
    }
}

impl GetRegister<u16> for u128 {
    #[inline(always)]
    fn get_register(&self) -> u16 {
        *self as u16
    }
}

impl GetRegister<u32> for u128 {
    #[inline(always)]
    fn get_register(&self) -> u32 {
        *self as u32
    }
}

impl GetRegister<u64> for u128 {
    #[inline(always)]
    fn get_register(&self) -> u64 {
        *self as u64
    }
}

impl GetRegister<u128> for u128 {
    #[inline(always)]
    fn get_register(&self) -> u128 {
        *self
    }
}
