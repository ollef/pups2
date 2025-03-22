use crate::bytes::Bytes;

pub struct Timer {
    timers: [TimerRegisters; 4],
    bus_clock: u64,
}

#[derive(Default, Clone)]
struct TimerRegisters {
    mode: u16,
    compare: u16,
    hold: u16,
    start: u64,
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
            0x10 => todo!(),
            0x20 => todo!(),
            0x30 => todo!(),
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
            0x10 => self.timers[timer].mode,
            0x20 => self.timers[timer].compare,
            0x30 => self.timers[timer].hold,
            _ => panic!("Invalid TIMER read at address: 0x{:08x}", address),
        }
    }

    pub fn step(&mut self) {
        self.bus_clock += 1;
    }
}
