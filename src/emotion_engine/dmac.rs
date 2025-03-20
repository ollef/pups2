use std::fmt::{Display, UpperHex};

use enum_map::{Enum, EnumMap};

use crate::bits::Bits;

use super::bus::{Bus, Bytes};

#[derive(Debug, Default)]
pub struct Dmac {
    control: ControlRegisterState, // CTRL
    status: u32,                   // STAT
    priority_control: u32,         // PCR
    skip_quad_word: u32,           // SQWC
    ring_buffer_size: u32,         // RBSR
    ring_buffer_offset: u32,       // RBOR
    stall_address: u32,            // STADR
    hold_status: u32,              // D_ENABLER (read-only)
    hold_control: u32,             // D_ENABLEW (write-only)
    channels: EnumMap<Channel, ChannelRegisters>,
}

#[derive(Debug, Default)]
struct ControlRegisterState {
    raw: u32,
}

impl ControlRegisterState {
    pub fn enabled(&self) -> bool {
        self.raw.bit(0)
    }

    pub fn released(&self) -> bool {
        self.raw.bit(1)
    }

    pub fn memory_fifo_drain_channel(&self) -> Option<Channel> {
        match self.raw.bits(2..4) {
            0b00 => None,
            0b01 => None, // Reserved
            0b10 => Some(Channel::Vif1),
            0b11 => Some(Channel::Gif),
            _ => unreachable!(),
        }
    }

    pub fn stall_control_source_channel(&self) -> Option<Channel> {
        match self.raw.bits(4..6) {
            0b00 => None,
            0b01 => Some(Channel::Sif0),
            0b10 => Some(Channel::Vif1),
            0b11 => Some(Channel::Gif),
            _ => unreachable!(),
        }
    }

    pub fn stall_control_drain_channel(&self) -> Option<Channel> {
        match self.raw.bits(6..8) {
            0b00 => None,
            0b01 => Some(Channel::Vif1),
            0b10 => Some(Channel::Gif),
            0b11 => Some(Channel::Sif1),
            _ => unreachable!(),
        }
    }

    pub fn release_cycle(&self) -> usize {
        match self.raw.bits(8..11) {
            0b000 => 8,
            0b001 => 16,
            0b010 => 32,
            0b011 => 64,
            0b100 => 128,
            _ => 256,
        }
    }
}

#[derive(Debug, Enum, Copy, Clone)]
pub enum Channel {
    Vif0,
    Vif1,
    Gif,
    FromIpu,
    ToIpu,
    Sif0,
    Sif1,
    Sif2,
    FromSpr,
    ToSpr,
}

#[derive(Debug, Default)]
pub struct ChannelRegisters {
    control: u32,                   // CHCR
    memory_address: u32,            // MADR
    quad_word_count: u32,           // QWC
    tag_address: u32,               // TADR
    tag_address_save_0: u32,        // ASR0
    tag_address_save_1: u32,        // ASR1
    scratchpad_memory_address: u32, // SADR
}

impl Dmac {
    pub fn write<T: Bytes + UpperHex>(&mut self, address: u32, value: T) {
        if std::mem::size_of::<T>() != 4 {
            panic!("Invalid DMAC write size: {}", std::mem::size_of::<T>());
        }
        println!("DMAC write address: 0x{:08X}=0x{:08X}", address, value);
        let value = u32::from_bytes(value.to_bytes().as_ref());
        // TODO: check which addresses can actually be written
        let channel = match address {
            0x1000_8000..0x1000_9000 => Channel::Vif0,
            0x1000_9000..0x1000_A000 => Channel::Vif1,
            0x1000_A000..0x1000_B000 => Channel::Gif,
            0x1000_B000..0x1000_B400 => Channel::FromIpu,
            0x1000_B400..0x1000_C000 => Channel::ToIpu,
            0x1000_C000..0x1000_C400 => Channel::Sif0,
            0x1000_C400..0x1000_C800 => Channel::Sif1,
            0x1000_C800..0x1000_D000 => Channel::Sif2,
            0x1000_D000..0x1000_D400 => Channel::FromSpr,
            0x1000_D400..0x1000_E000 => Channel::ToSpr,
            0x1000_E000 => {
                self.control.raw = value;
                return;
            }
            0x1000_E010 => {
                self.status = value;
                return;
            }
            0x1000_E020 => {
                self.priority_control = value;
                return;
            }
            0x1000_E030 => {
                self.skip_quad_word = value;
                return;
            }
            0x1000_E040 => {
                self.ring_buffer_size = value;
                return;
            }
            0x1000_E050 => {
                self.ring_buffer_offset = value;
                return;
            }
            0x1000_E060 => {
                self.stall_address = value;
                return;
            }
            0x1000_F590 => {
                self.hold_control = value;
                return;
            }
            _ => panic!("Invalid DMAC write address: 0x{:08X}", address),
        };
        match address & 0xFF {
            0x00 => self.channels[channel].control = value,
            0x10 => self.channels[channel].memory_address = value,
            0x20 => self.channels[channel].quad_word_count = value,
            0x30 => self.channels[channel].tag_address = value,
            0x40 => self.channels[channel].tag_address_save_0 = value,
            0x50 => self.channels[channel].tag_address_save_1 = value,
            0x80 => self.channels[channel].scratchpad_memory_address = value,
            _ => panic!("Invalid write to DMAC: 0x{:08X} {}", address, value),
        }
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        if std::mem::size_of::<T>() != 4 {
            panic!("Invalid DMAC read size: {}", std::mem::size_of::<T>());
        }
        println!("DMAC read address: 0x{:08X}", address);
        // TODO: check which addresses can actually be read
        let channel = match address {
            0x1000_8000..0x1000_9000 => Channel::Vif0,
            0x1000_9000..0x1000_A000 => Channel::Vif1,
            0x1000_A000..0x1000_B000 => Channel::Gif,
            0x1000_B000..0x1000_B400 => Channel::FromIpu,
            0x1000_B400..0x1000_C000 => Channel::ToIpu,
            0x1000_C000..0x1000_C400 => Channel::Sif0,
            0x1000_C400..0x1000_C800 => Channel::Sif1,
            0x1000_C800..0x1000_D000 => Channel::Sif2,
            0x1000_D000..0x1000_D400 => Channel::FromSpr,
            0x1000_D400..0x1000_E000 => Channel::ToSpr,
            0x1000_E000 => return T::from_bytes(&self.control.raw.to_bytes()),
            0x1000_E010 => return T::from_bytes(&self.status.to_bytes()),
            0x1000_E020 => return T::from_bytes(&self.priority_control.to_bytes()),
            0x1000_E030 => return T::from_bytes(&self.skip_quad_word.to_bytes()),
            0x1000_E040 => return T::from_bytes(&self.ring_buffer_size.to_bytes()),
            0x1000_E050 => return T::from_bytes(&self.ring_buffer_offset.to_bytes()),
            0x1000_E060 => return T::from_bytes(&self.stall_address.to_bytes()),
            0x1000_F520 => return T::from_bytes(&self.hold_status.to_bytes()),
            _ => panic!("Invalid DMAC read address: 0x{:08X}", address),
        };
        match address & 0xFF {
            0x00 => T::from_bytes(&self.channels[channel].control.to_bytes()),
            0x10 => T::from_bytes(&self.channels[channel].memory_address.to_bytes()),
            0x20 => T::from_bytes(&self.channels[channel].quad_word_count.to_bytes()),
            0x30 => T::from_bytes(&self.channels[channel].tag_address.to_bytes()),
            0x40 => T::from_bytes(&self.channels[channel].tag_address_save_0.to_bytes()),
            0x50 => T::from_bytes(&self.channels[channel].tag_address_save_1.to_bytes()),
            0x80 => T::from_bytes(&self.channels[channel].scratchpad_memory_address.to_bytes()),
            _ => panic!("Invalid read from DMAC: 0x{:08X}", address),
        }
    }

    pub fn step(bus: &mut Bus) {
        for (channel, registers) in &mut bus.dmac.channels {}
    }
}
