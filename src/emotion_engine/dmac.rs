use enum_map::{Enum, EnumMap};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes};

use super::bus::Bus;

#[derive(Debug, Default)]
pub struct Dmac {
    control: ControlRegister, // CTRL
    status: StatusRegister,   // STAT
    priority_control: u32,    // PCR
    skip_quad_word: u32,      // SQWC
    ring_buffer_size: u32,    // RBSR
    ring_buffer_offset: u32,  // RBOR
    stall_address: u32,       // STADR
    hold_status: u32,         // D_ENABLER (read-only)
    hold_control: u32,        // D_ENABLEW (write-only)
    channels: EnumMap<Channel, ChannelRegisters>,
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

impl Channel {
    pub fn all() -> impl ExactSizeIterator<Item = Channel> {
        (0..Channel::LENGTH).map(Channel::from_usize)
    }
}

#[derive(Debug, Default)]
pub struct ChannelRegisters {
    control: ChannelControlRegister,               // CHCR
    memory_address: MemoryOrScratchpadAddress,     // MADR
    quad_word_count: u32,                          // QWC
    tag_address: MemoryOrScratchpadAddress,        // TADR
    tag_address_save_0: MemoryOrScratchpadAddress, // ASR0
    tag_address_save_1: MemoryOrScratchpadAddress, // ASR1
    scratchpad_memory_address: u32,                // SADR
    process_next_tag: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MemoryOrScratchpadAddress(u32);

pub enum MemoryOrScratchpadAddressView {
    Memory(u32),
    Scratchpad(u32),
}

impl MemoryOrScratchpadAddress {
    pub fn view(&self) -> MemoryOrScratchpadAddressView {
        if self.0.bit(31) {
            MemoryOrScratchpadAddressView::Scratchpad(self.0.bits(0..31))
        } else {
            MemoryOrScratchpadAddressView::Memory(self.0.bits(0..31))
        }
    }
}

impl Dmac {
    pub fn write<T: Bytes>(&mut self, address: u32, value: T) {
        match std::mem::size_of::<T>() {
            4 => self.write32(address, u32::from_bytes(value.to_bytes().as_ref())),
            _ => panic!("Invalid write size {}", std::mem::size_of::<T>()),
        }
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        match std::mem::size_of::<T>() {
            4 => T::from_bytes(self.read32(address).to_bytes().as_ref()),
            _ => panic!("Invalid read size {}", std::mem::size_of::<T>()),
        }
    }

    pub fn write32(&mut self, address: u32, value: u32) {
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
                self.status.write(StatusRegister { raw: value });
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
            _ => panic!("Invalid DMAC write address: 0x{:08x}", address),
        };
        match address & 0xFF {
            0x00 => {
                self.channels[channel].control.raw = value;
                if self.channels[channel].control.start() {
                    self.channels[channel].process_next_tag = true;
                }
            }
            0x10 => self.channels[channel].memory_address = MemoryOrScratchpadAddress(value),
            0x20 => self.channels[channel].quad_word_count = value,
            0x30 => self.channels[channel].tag_address = MemoryOrScratchpadAddress(value),
            0x40 => self.channels[channel].tag_address_save_0 = MemoryOrScratchpadAddress(value),
            0x50 => self.channels[channel].tag_address_save_1 = MemoryOrScratchpadAddress(value),
            0x80 => self.channels[channel].scratchpad_memory_address = value.bits(0..=13),
            _ => panic!("Invalid write to DMAC: 0x{:08x} {}", address, value),
        }
    }

    pub fn read32(&self, address: u32) -> u32 {
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
            0x1000_E000 => return self.control.raw,
            0x1000_E010 => return self.status.raw,
            0x1000_E020 => return self.priority_control,
            0x1000_E030 => return self.skip_quad_word,
            0x1000_E040 => return self.ring_buffer_size,
            0x1000_E050 => return self.ring_buffer_offset,
            0x1000_E060 => return self.stall_address,
            0x1000_F520 => return self.hold_status,
            _ => panic!("Invalid DMAC read address: 0x{:08x}", address),
        };
        match address & 0xFF {
            0x00 => self.channels[channel].control.raw,
            0x10 => self.channels[channel].memory_address.0,
            0x20 => self.channels[channel].quad_word_count,
            0x30 => self.channels[channel].tag_address.0,
            0x40 => self.channels[channel].tag_address_save_0.0,
            0x50 => self.channels[channel].tag_address_save_1.0,
            0x80 => self.channels[channel].scratchpad_memory_address,
            _ => panic!("Invalid read from DMAC: 0x{:08x}", address),
        }
    }

    pub fn step(bus: &mut Bus) {
        // TODO arbitration
        for channel in Channel::all() {
            let registers = &bus.dmac.channels[channel];
            if registers.control.start() {
                match channel {
                    Channel::Vif0 => todo!(),
                    Channel::Vif1 => todo!(),
                    Channel::Gif => match registers.control.mode() {
                        ChannelMode::Normal => {
                            let mut memory_address = registers.memory_address;
                            let mut quad_word_count = registers.quad_word_count;
                            while quad_word_count > 0 && !bus.gif.fifo.is_full() {
                                let data = bus.read_memory_or_scratchpad::<u128>(memory_address);
                                bus.gif.fifo.push_back(data);
                                // println!(
                                //     "Transferred quad word 0x{:08x} from 0x{:08x} to GIF FIFO (QWC={})",
                                //     data,
                                //     memory_address, quad_word_count
                                // );
                                memory_address.0 += 16;
                                quad_word_count -= 1;
                            }
                            let registers = &mut bus.dmac.channels[channel];
                            if quad_word_count == 0 {
                                registers.control.set_start(false);
                                // println!(
                                //     "GIF channel finished, control=0x{:08x}",
                                //     registers.control.raw
                                // );
                            }
                            registers.memory_address = memory_address;
                            registers.quad_word_count = quad_word_count;
                        }
                        ChannelMode::Chain => {
                            let mut memory_address = registers.memory_address;
                            let mut quad_word_count = registers.quad_word_count;
                            while quad_word_count > 0 && !bus.gif.fifo.is_full() {
                                let data = bus.read_memory_or_scratchpad::<u128>(memory_address);
                                bus.gif.fifo.push_back(data);
                                // println!(
                                //     "Transferred quad word 0x{:08x} from 0x{:08x} to GIF FIFO (QWC={})",
                                //     data,
                                //     memory_address, quad_word_count
                                // );
                                memory_address.0 += 16;
                                quad_word_count -= 1;
                            }
                            if quad_word_count == 0 {
                                if registers.process_next_tag {
                                    assert!(!registers.control.tag_transfer_enable());
                                    let source_chain_tag = bus
                                        .read_memory_or_scratchpad::<u128>(registers.tag_address);
                                    let registers = &mut bus.dmac.channels[channel];
                                    registers
                                        .control
                                        .set_dma_tag(source_chain_tag.bits(16..32) as u16);
                                    let source_chain_tag =
                                        SourceChainTag::from(source_chain_tag as u64);
                                    quad_word_count = source_chain_tag.quad_word_count as u32;
                                    match source_chain_tag.tag_id {
                                        TagId::ReferenceEnd => {
                                            memory_address = source_chain_tag.address;
                                            registers.tag_address.0 += 16;
                                            registers.process_next_tag = false;
                                        }
                                        TagId::Count => {
                                            memory_address = registers.tag_address;
                                            memory_address.0 += 16;
                                            registers.tag_address = source_chain_tag.address;
                                        }
                                        TagId::Next => todo!(),
                                        TagId::Reference => todo!(),
                                        TagId::References => todo!(),
                                        TagId::Call => todo!(),
                                        TagId::Return => todo!(),
                                        TagId::End => todo!(),
                                    }
                                } else {
                                    let registers = &mut bus.dmac.channels[channel];
                                    registers.control.set_start(false);
                                    // println!(
                                    //     "GIF channel finished, control=0x{:08x}",
                                    //     registers.control.raw
                                    // );
                                }
                            }
                            let registers = &mut bus.dmac.channels[channel];
                            registers.memory_address = memory_address;
                            registers.quad_word_count = quad_word_count;
                        }
                        ChannelMode::Interleave => todo!(),
                    },
                    Channel::FromIpu => todo!(),
                    Channel::ToIpu => todo!(),
                    Channel::Sif0 => todo!(),
                    Channel::Sif1 => todo!(),
                    Channel::Sif2 => todo!(),
                    Channel::FromSpr => todo!(),
                    Channel::ToSpr => todo!(),
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct ControlRegister {
    raw: u32,
}

impl ControlRegister {
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

#[derive(Debug, Default, Copy, Clone)]
struct StatusRegister {
    raw: u32,
}

impl StatusRegister {
    pub fn interrupt_status(self, channel: Channel) -> bool {
        self.raw.bit(channel.into_usize())
    }

    pub fn set_interrupt_status(&mut self, channel: Channel, value: bool) {
        self.raw.set_bit(channel.into_usize(), value);
    }

    pub fn dma_stall_interrupt_status(self) -> bool {
        self.raw.bit(13)
    }

    pub fn set_dma_stall_interrupt_status(&mut self, value: bool) {
        self.raw.set_bit(13, value);
    }

    pub fn mfifo_empty_interrupt_status(self) -> bool {
        self.raw.bit(14)
    }

    pub fn set_mfifo_empty_interrupt_status(&mut self, value: bool) {
        self.raw.set_bit(14, value);
    }

    pub fn buserr_interrupt_status(self) -> bool {
        self.raw.bit(15)
    }

    pub fn set_buserr_interrupt_status(&mut self, value: bool) {
        self.raw.set_bit(15, value);
    }

    pub fn interrupt_mask(self, channel: Channel) -> bool {
        self.raw.bit(channel.into_usize() + 16)
    }

    pub fn set_interrupt_mask(&mut self, channel: Channel, value: bool) {
        self.raw.set_bit(channel.into_usize() + 16, value);
    }

    pub fn dma_stall_interrupt_mask(self) -> bool {
        self.raw.bit(29)
    }

    pub fn set_dma_stall_interrupt_mask(&mut self, value: bool) {
        self.raw.set_bit(29, value);
    }

    pub fn mfifo_empty_interrupt_mask(self) -> bool {
        self.raw.bit(30)
    }

    pub fn set_mfifo_empty_interrupt_mask(&mut self, value: bool) {
        self.raw.set_bit(30, value);
    }

    pub fn write(&mut self, value: Self) {
        for channel in Channel::all() {
            if value.interrupt_status(channel) {
                self.set_interrupt_status(channel, false)
            }
            if value.interrupt_mask(channel) {
                self.set_interrupt_mask(channel, !self.interrupt_mask(channel))
            }
        }
        if value.dma_stall_interrupt_status() {
            self.set_dma_stall_interrupt_status(false)
        }
        if value.dma_stall_interrupt_mask() {
            self.set_dma_stall_interrupt_mask(!self.dma_stall_interrupt_mask())
        }
        if value.mfifo_empty_interrupt_status() {
            self.set_mfifo_empty_interrupt_status(false)
        }
        if value.mfifo_empty_interrupt_mask() {
            self.set_mfifo_empty_interrupt_mask(!self.mfifo_empty_interrupt_mask())
        }
        if value.buserr_interrupt_status() {
            self.set_buserr_interrupt_status(false)
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ChannelControlRegister {
    raw: u32,
}

impl ChannelControlRegister {
    pub fn direction(self) -> ChannelDirection {
        ChannelDirection::from_u32(self.raw.bits(0..=0))
            .unwrap_or_else(|| panic!("Invalid DMAC channel direction: {}", self.raw.bits(0..=0)))
    }

    pub fn mode(self) -> ChannelMode {
        ChannelMode::from_u32(self.raw.bits(2..=3))
            .unwrap_or_else(|| panic!("Invalid DMAC channel mode: {}", self.raw.bits(2..=3)))
    }

    pub fn address_stack_pointer(self) -> u32 {
        self.raw.bits(4..=5)
    }

    pub fn tag_transfer_enable(self) -> bool {
        self.raw.bit(6)
    }

    pub fn tag_interrupt_enable(self) -> bool {
        self.raw.bit(7)
    }

    pub fn start(self) -> bool {
        self.raw.bit(8)
    }

    pub fn set_start(&mut self, value: bool) {
        self.raw.set_bit(8, value);
    }

    pub fn dma_tag(self) -> u16 {
        self.raw.bits(16..) as u16
    }

    pub fn set_dma_tag(&mut self, value: u16) {
        self.raw.set_bits(16.., value as u32);
    }
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum ChannelDirection {
    ToMemory = 0b0,
    FromMemory = 0b1,
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum ChannelMode {
    Normal = 0b00,
    Chain = 0b01,
    Interleave = 0b10,
}

struct SourceChainTag {
    quad_word_count: u16,               // QWC
    priority_control: PriorityControl,  // PCE
    tag_id: TagId,                      // ID
    interrupt_request: bool,            // IRQ
    address: MemoryOrScratchpadAddress, // ADDR, SPR
}

impl From<u64> for SourceChainTag {
    fn from(raw: u64) -> Self {
        Self {
            quad_word_count: raw.bits(0..=15) as u16,
            priority_control: PriorityControl::from_u64(raw.bits(26..=27))
                .unwrap_or_else(|| panic!("Invalid DMAC priority control: {}", raw.bits(26..=27))),
            tag_id: TagId::from_u64(raw.bits(28..=30))
                .unwrap_or_else(|| panic!("Invalid DMAC tag ID: {}", raw.bits(28..=30))),
            interrupt_request: raw.bit(31),
            address: MemoryOrScratchpadAddress(raw.bits(32..64) as u32),
        }
    }
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum PriorityControl {
    Nothing = 0b00,
    Reserved = 0b01,
    Disabled = 0b10,
    Enabled = 0b11,
}

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum TagId {
    ReferenceEnd = 0b000, // refe
    Count = 0b001,        // cnt
    Next = 0b010,         // next
    Reference = 0b011,    // ref
    References = 0b100,   // refs
    Call = 0b101,         // call
    Return = 0b110,       // ret
    End = 0b111,          // ret
}
