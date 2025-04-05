use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes};

use super::{registers::PixelStorageFormat, Gs};

#[derive(Debug, Default)]
pub struct PrivilegedRegisters {
    pub pcrtc_mode: u64,                           // PMODE
    pub sync_mode1: u64,                           // SMODE1
    pub sync_mode2: u64,                           // SMODE2
    pub dram_refresh: u64,                         // SRFSH
    pub synch1: u64,                               // SYNCH1
    pub synch2: u64,                               // SYNCH2
    pub syncv: u64,                                // SYNCV
    pub display_frame_buffer1: DisplayFrameBuffer, // DISPFB1
    pub display1: Display,                         // DISPLAY1
    pub display_frame_buffer2: DisplayFrameBuffer, // DISPFB1
    pub display2: Display,                         // DISPLAY1
    pub write_buffer: u64,                         // EXTBUF
    pub write_data: u64,                           // EXTDATA
    pub write_start: u64,                          // EXTWRITE
    pub background_color: u64,                     // BGCOLOR
    pub status: u64,                               // CSR
    pub interrupt_mask: u64,                       // IMR
    pub bus_direction: u64,                        // BUSDIR
    pub signal_label_id: u64,                      // SIGLBLID
}

#[derive(Debug, Default)]
pub struct DisplayFrameBuffer {
    pub base_pointer: u32,
    pub width: u16,
    pub pixel_storage_format: PixelStorageFormat,
    pub offset_x: u16,
    pub offset_y: u16,
}

impl From<u64> for DisplayFrameBuffer {
    fn from(raw: u64) -> Self {
        DisplayFrameBuffer {
            base_pointer: raw.bits(0..=8) as u32 * 2048,
            width: raw.bits(9..=14) as u16 * 64,
            pixel_storage_format: PixelStorageFormat::from_u64(raw.bits(15..=19))
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(24..=29))),
            offset_x: raw.bits(32..=42) as u16,
            offset_y: raw.bits(43..=53) as u16,
        }
    }
}

#[derive(Debug, Default)]
pub struct Display {
    x_position: u16,              // DX
    y_position: u16,              // DX
    horizontal_magnification: u8, // MAGH
    vertical_magnification: u8,   // MAGV
    width: u16,                   // DW
    height: u16,                  // DH
}

impl From<u64> for Display {
    fn from(raw: u64) -> Self {
        let horizontal_magnification = raw.bits(23..=26) as u8 + 1;
        let vertical_magnification = raw.bits(27..=28) as u8 + 1;
        Display {
            x_position: raw.bits(0..=11) as u16 / horizontal_magnification as u16,
            y_position: raw.bits(12..=22) as u16 / vertical_magnification as u16,
            horizontal_magnification,
            vertical_magnification,
            width: (raw.bits(32..=43) as u16 + 1) / horizontal_magnification as u16,
            height: (raw.bits(44..=54) as u16 + 1) / vertical_magnification as u16,
        }
    }
}

impl Gs {
    pub fn write_privileged<T: Bytes>(&mut self, address: u32, value: T) {
        match std::mem::size_of::<T>() {
            4 => {
                let aligned_address = address & !0b111;
                let offset_bytes = (address - aligned_address) as usize;
                let existing = self.read_privileged64(aligned_address);
                let value =
                    (u32::from_bytes(value.to_bytes().as_ref()) as u64) << (offset_bytes * 8);
                let value = existing & !u64::mask(offset_bytes * 8..offset_bytes * 8 + 32) | value;
                self.write_privileged64(aligned_address, value);
            }
            8 => self.write_privileged64(address, u64::from_bytes(value.to_bytes().as_ref())),
            _ => panic!("Invalid GS write size: {}", std::mem::size_of::<T>()),
        }
    }
    pub fn read_privileged<T: Bytes>(&self, address: u32) -> T {
        match std::mem::size_of::<T>() {
            4 => {
                let aligned_address = address & !0b111;
                let result = self.read_privileged64(aligned_address);
                let offset_bytes = (address - aligned_address) as usize;
                T::from_bytes(&result.to_bytes()[offset_bytes..offset_bytes + 4])
            }
            8 => T::from_bytes(self.read_privileged64(address).to_bytes().as_ref()),
            _ => panic!(
                "Invalid privileged GS read size: {} to {address:08x}",
                std::mem::size_of::<T>()
            ),
        }
    }

    pub fn write_privileged64(&mut self, address: u32, value: u64) {
        match address {
            0x1200_0000 => self.privileged_registers.pcrtc_mode = value,
            0x1200_0010 => self.privileged_registers.sync_mode1 = value,
            0x1200_0020 => self.privileged_registers.sync_mode2 = value,
            0x1200_0030 => self.privileged_registers.dram_refresh = value,
            0x1200_0040 => self.privileged_registers.synch1 = value,
            0x1200_0050 => self.privileged_registers.synch2 = value,
            0x1200_0060 => self.privileged_registers.syncv = value,
            0x1200_0070 => {
                self.privileged_registers.display_frame_buffer1 = DisplayFrameBuffer::from(value);
                println!(
                    "Display frame buffer 1 = {:?}",
                    self.privileged_registers.display_frame_buffer1
                )
            }
            0x1200_0080 => {
                self.privileged_registers.display1 = Display::from(value);
                println!("Display 1 = {:?}", self.privileged_registers.display1)
            }
            0x1200_0090 => {
                self.privileged_registers.display_frame_buffer2 = DisplayFrameBuffer::from(value);
                println!(
                    "Display frame buffer 2 = {:?}",
                    self.privileged_registers.display_frame_buffer2
                )
            }
            0x1200_00A0 => {
                self.privileged_registers.display2 = Display::from(value);
                println!("Display 2 = {:?}", self.privileged_registers.display2)
            }
            0x1200_00B0 => self.privileged_registers.write_buffer = value,
            0x1200_00C0 => self.privileged_registers.write_data = value,
            0x1200_00D0 => self.privileged_registers.write_start = value,
            0x1200_00E0 => self.privileged_registers.background_color = value,
            0x1200_1000 => self.privileged_registers.status = value,
            0x1200_1010 => self.privileged_registers.interrupt_mask = value,
            0x1200_1040 => self.privileged_registers.bus_direction = value,
            0x1200_1080 => self.privileged_registers.signal_label_id = value,
            _ => panic!("Invalid GS write64 {} to address: 0x{:08x}", value, address),
        }
    }

    pub fn read_privileged64(&self, address: u32) -> u64 {
        match address {
            0x1200_1000 => self.privileged_registers.status,
            0x1200_1080 => self.privileged_registers.signal_label_id,
            _ => panic!("Invalid GS read64 from address: 0x{:08x}", address),
        }
    }
}
