use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes};

pub struct Timer {
    timers: [TimerRegisters; 4],
    bus_clock: u64,
}

#[derive(Default, Clone)]
struct TimerRegisters {
    mode: Mode,
    compare: u16,
    hold: u16,
    start: u64,
}

#[derive(Default, Clone, Copy)]
struct Mode(u16);

impl Mode {
    // CLKS
    pub fn selection(self) -> ClockSelection {
        ClockSelection::from_u16(self.0.bits(0..=1)).unwrap()
    }

    // GATE
    pub fn gate(self) -> bool {
        self.0.bit(2)
    }

    // GATS
    pub fn gate_selection(self) -> GateSelection {
        GateSelection::from_u16(self.0.bits(3..=3)).unwrap()
    }

    // GATM
    pub fn gate_mode(self) -> GateMode {
        GateMode::from_u16(self.0.bits(4..=5)).unwrap()
    }

    // ZRET
    pub fn zero_return(self) -> bool {
        self.0.bit(6)
    }

    // CUE
    pub fn count_up_enable(self) -> bool {
        self.0.bit(7)
    }

    // CMPE
    pub fn compare_interrupt_enable(self) -> bool {
        self.0.bit(8)
    }

    // OVFE
    pub fn overflow_interrupt_enable(self) -> bool {
        self.0.bit(9)
    }

    // EQUF
    pub fn equal_flag(self) -> bool {
        self.0.bit(10)
    }

    pub fn set_equal_flag(&mut self, value: bool) {
        self.0.set_bit(10, value);
    }

    pub fn overflow_flag(self) -> bool {
        self.0.bit(11)
    }

    pub fn set_overflow_flag(&mut self, value: bool) {
        self.0.set_bit(11, value);
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
enum ClockSelection {
    BusClock = 0b00,
    BusClockDiv16 = 0b01,
    BusClockDiv256 = 0b10,
    HBlank = 0b11,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
enum GateSelection {
    HBlank = 0b0,
    VBlank = 0b1,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
enum GateMode {
    SignalLow = 0b00,
    ResetOnRisingEdge = 0b01,
    ResetOnFallingEdge = 0b10,
    ResetOnBothEdges = 0b11,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            timers: [
                TimerRegisters::default(),
                TimerRegisters::default(),
                TimerRegisters::default(),
                TimerRegisters::default(),
            ],
            bus_clock: 0,
        }
    }

    pub fn write<T: Bytes>(&mut self, address: u32, value: T) {
        match std::mem::size_of::<T>() {
            4 => self.write16(address, u32::from_bytes(value.to_bytes().as_ref()) as u16),
            _ => panic!("Invalid write size {}", std::mem::size_of::<T>()),
        }
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        match std::mem::size_of::<T>() {
            4 => T::from_bytes((self.read16(address) as u32).to_bytes().as_ref()),
            _ => panic!("Invalid read size {}", std::mem::size_of::<T>()),
        }
    }

    pub fn write16(&mut self, address: u32, value: u16) {
        let value = u16::from_bytes(value.to_bytes().as_ref());
        let timer = match address {
            0x1000_0000..0x1000_0800 => 0,
            0x1000_0800..0x1000_1000 => 1,
            0x1000_1000..0x1000_1800 => 2,
            0x1000_1800..0x1000_2000 => 3,
            _ => panic!(
                "Invalid TIMER write of {} at address: 0x{:08x}",
                value, address
            ),
        };
        match address & 0xFF {
            0x00 => todo!(),
            0x10 => {
                let mut mode = Mode(value);
                mode.set_equal_flag(false);
                mode.set_overflow_flag(false);
                self.timers[timer].mode = mode;
                println!("Timer {} mode: 0x{:04b}", timer, value);
            }
            0x20 => todo!(),
            0x30 if timer == 0 || timer == 1 => todo!(),
            _ => panic!(
                "Invalid TIMER write of {} at address: 0x{:08x}",
                value, address
            ),
        }
    }

    pub fn read16(&self, address: u32) -> u16 {
        let timer = match address {
            0x1000_0000..0x1000_0800 => 0,
            0x1000_0800..0x1000_1000 => 1,
            0x1000_1000..0x1000_1800 => 2,
            0x1000_1800..0x1000_2000 => 3,
            _ => panic!("Invalid TIMER read at address: 0x{:08x}", address),
        };
        match address & 0xFF {
            0x00 => (self.bus_clock - self.timers[timer].start) as u16,
            0x10 => self.timers[timer].mode.0,
            0x20 => self.timers[timer].compare,
            0x30 if timer == 0 || timer == 1 => self.timers[timer].hold,
            _ => panic!("Invalid TIMER read at address: 0x{:08x}", address),
        }
    }

    pub fn step(&mut self) {
        self.bus_clock += 1;
    }
}
