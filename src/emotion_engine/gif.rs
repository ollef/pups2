use enum_map::Enum;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes, fifo::Fifo};

use super::{bus::Bus, gs};

pub struct Gif {
    pub fifo: Fifo<u128>,
    control: u32,                       // CTRL
    mode: u32,                          // MODE
    status: u32,                        // STAT
    tag: Tag,                           // TAG0, TAG1, TAG2, TAG3
    transfer_status: TransferStatus,    // CNT
    path3_transfer_status_counter: u32, // P3CNT
    path3_tag_value: u32,               // P3TAG
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum)]
#[repr(u8)]
pub enum Register {
    Primitive,
    Rgbaq,
    St,
    Uv,
    Xyzf2,
    Xyz2,
    Tex01,
    Tex02,
    Clamp1,
    Clamp2,
    Fog,
    Reserved,
    Xyzf3,
    Xyz3,
    AddressData,
    Nop,
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        let value = value & 0b1111;
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Debug, Clone)]
pub struct Tag {
    raw: u128,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Enum)]
pub enum DataFormat {
    Packed,
    RegisterList,
    Image,
}

impl Tag {
    // NLOOP
    pub fn repeat_count(&self) -> u16 {
        self.raw.bits(0..=14) as u16
    }

    // EOP
    pub fn end_of_packet(&self) -> bool {
        self.raw.bit(15)
    }

    // PRE
    pub fn prim_field_enable(&self) -> bool {
        self.raw.bit(46)
    }

    // PRIM
    pub fn prim_data(&self) -> u16 {
        self.raw.bits(47..=57) as u16
    }

    // FLG
    pub fn data_format(&self) -> DataFormat {
        match self.raw.bits(58..=59) as u8 {
            0b00 => DataFormat::Packed,
            0b01 => DataFormat::RegisterList,
            0b10 => DataFormat::Image,
            0b11 => DataFormat::Image,
            _ => unreachable!(),
        }
    }

    // NREG
    pub fn register_count(&self) -> u8 {
        match self.raw.bits(60..=63) as u8 {
            0 => 16,
            n => n,
        }
    }

    pub fn register(&self, index: u8) -> Register {
        let start = 64 + index * 4;
        let end = start + 4;
        Register::from(self.raw.bits(start..end) as u8)
    }

    // REGS
    pub fn registers(&self) -> impl ExactSizeIterator<Item = Register> + '_ {
        (0..self.register_count()).map(|i| self.register(i))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TransferStatus {
    raw: u32,
}

impl TransferStatus {
    pub fn loop_counter(self) -> u16 {
        self.raw.bits(0..=14) as u16
    }

    pub fn set_loop_counter(&mut self, value: u16) {
        self.raw.set_bits(0..=14, value)
    }

    pub fn register_counter(self) -> u8 {
        self.raw.bits(16..=19) as u8
    }

    pub fn set_register_counter(&mut self, value: u8) {
        self.raw.set_bits(16..=19, value)
    }

    pub fn vu_address(self) -> u16 {
        self.raw.bits(20..=29) as u16
    }
}

impl Gif {
    pub fn new() -> Gif {
        Gif {
            fifo: Fifo::with_capacity(16),
            control: 0,
            mode: 0,
            status: 0,
            tag: Tag { raw: 0 },
            transfer_status: TransferStatus { raw: 0 },
            path3_transfer_status_counter: 0,
            path3_tag_value: 0,
        }
    }

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
        match address {
            0x1000_3000 => self.control = value,
            0x1000_3010 => self.mode = value,
            _ => panic!(
                "Invalid GIF write of {} at address: 0x{:08x}",
                value, address
            ),
        }
    }

    pub fn read32(&self, address: u32) -> u32 {
        match address {
            0x1000_3020 => self.status,
            0x1000_3040 => self.tag.raw.bits(0..32) as u32,
            0x1000_3050 => self.tag.raw.bits(32..64) as u32,
            0x1000_3060 => self.tag.raw.bits(64..96) as u32,
            0x1000_3070 => self.tag.raw.bits(96..128) as u32,
            0x1000_3080 => self.transfer_status.raw,
            0x1000_3090 => self.path3_transfer_status_counter,
            0x1000_30a0 => self.path3_tag_value,
            _ => panic!("Invalid GIF read at address: 0x{:08x}", address),
        }
    }

    pub fn step(bus: &mut Bus) {
        while let Some(data) = bus.gif.fifo.pop_front() {
            // println!("FIFO data = {:08x}", data);
            let loop_counter = bus.gif.transfer_status.loop_counter();
            let mut register_counter = bus.gif.transfer_status.register_counter();
            if loop_counter == 0 && register_counter == 0 {
                let tag = Tag { raw: data };
                // println!("GIF tag: {:?}", tag);
                // println!("format: {:?}", tag.data_format());
                // println!("repeat count: {:?}", tag.repeat_count());
                // println!("end of packet: {:?}", tag.end_of_packet());
                // println!("register count: {:?}", tag.register_count());
                if tag.prim_field_enable() {
                    // println!("GIF tag write to prim: {:?}", tag.prim_data());
                }
                // print!("Registers:");
                for register in tag.registers() {
                    // print!(" {:?}", register);
                }
                println!();
                bus.gif.transfer_status.set_loop_counter(tag.repeat_count());
                bus.gif.tag = tag;
                continue;
            }

            match bus.gif.tag.data_format() {
                DataFormat::Packed => {
                    let register = bus.gif.tag.register(register_counter);
                    match register {
                        Register::Primitive => {
                            // println!("GIF write to prim: {:08x}", data.bits(0..=10));
                            bus.gs
                                .command_queue
                                .push_back((gs::Register::Primitive, data.bits(0..=10) as u64));
                        }
                        Register::Rgbaq => todo!(),
                        Register::St => todo!(),
                        Register::Uv => todo!(),
                        Register::Xyzf2 => {
                            let x = data.bits(0..=15) as u64;
                            let y = data.bits(32..=47) as u64;
                            let z = data.bits(68..=91) as u64;
                            let f = data.bits(100..=107) as u64;
                            let adc = data.bit(111);
                            bus.gs.command_queue.push_back((
                                if adc {
                                    gs::Register::Xyzf3
                                } else {
                                    gs::Register::Xyzf2
                                },
                                x | (y << 16) | (z << 32) | (f << 56),
                            ));
                        }
                        Register::Xyz2 => {
                            let x = data.bits(0..=15) as u64;
                            let y = data.bits(32..=47) as u64;
                            let z = data.bits(64..=95) as u64;
                            let adc = data.bit(111);
                            bus.gs.command_queue.push_back((
                                if adc {
                                    gs::Register::Xyz3
                                } else {
                                    gs::Register::Xyz2
                                },
                                x | (y << 16) | (z << 32),
                            ));
                        }
                        Register::Tex01 => todo!(),
                        Register::Tex02 => todo!(),
                        Register::Clamp1 => todo!(),
                        Register::Clamp2 => todo!(),
                        Register::Fog => todo!(),
                        Register::Reserved => todo!(),
                        Register::Xyzf3 => todo!(),
                        Register::Xyz3 => todo!(),
                        Register::AddressData => {
                            let register = gs::Register::from_u8(data.bits(64..=71) as u8)
                                .expect("Invalid GS register");
                            bus.gs
                                .command_queue
                                .push_back((register, data.bits(0..64) as u64));
                            // println!(
                            //     "GIF write address data: {:08x}={:08x}",
                            //     data.bits(64..=71),
                            //     data.bits(0..64)
                            // );
                        }
                        Register::Nop => todo!(),
                    }
                    register_counter += 1;
                    if register_counter == bus.gif.tag.register_count() {
                        register_counter = 0;
                        bus.gif.transfer_status.set_loop_counter(loop_counter - 1);
                        // println!("Decrementing loop counter = {}", loop_counter - 1);
                    }
                    bus.gif
                        .transfer_status
                        .set_register_counter(register_counter);
                }
                DataFormat::RegisterList => todo!(),
                DataFormat::Image => {
                    bus.gs
                        .command_queue
                        .push_back((gs::Register::TransmissionData, data.bits(0..64) as u64));
                    bus.gs
                        .command_queue
                        .push_back((gs::Register::TransmissionData, data.bits(64..128) as u64));
                    bus.gif.transfer_status.set_loop_counter(loop_counter - 1);
                    // println!("Decrementing loop counter = {}", loop_counter - 1);
                }
            }
        }
    }
}
