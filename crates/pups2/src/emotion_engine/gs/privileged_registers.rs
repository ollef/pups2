use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes};

use super::{registers::PixelStorageFormat, rendering::Rect, Gs};

#[derive(Debug, Default)]
pub struct PrivilegedRegisters {
    pub pcrtc_mode: PcrtcMode,                     // PMODE
    pub sync_mode1: u64,                           // SMODE1
    pub sync_mode2: u64,                           // SMODE2
    pub dram_refresh: u64,                         // SRFSH
    pub synch1: u64,                               // SYNCH1
    pub synch2: u64,                               // SYNCH2
    pub syncv: u64,                                // SYNCV
    pub display_frame_buffer1: DisplayFrameBuffer, // DISPFB1
    pub display1: Display,                         // DISPLAY1
    pub display_frame_buffer2: DisplayFrameBuffer, // DISPFB1
    pub display2: Display,                         // DISPLAY2
    pub write_buffer: u64,                         // EXTBUF
    pub write_data: u64,                           // EXTDATA
    pub write_start: u64,                          // EXTWRITE
    pub background_color: Rgb,                     // BGCOLOR
    pub status: u64,                               // CSR
    pub interrupt_mask: u64,                       // IMR
    pub bus_direction: u64,                        // BUSDIR
    pub signal_label_id: u64,                      // SIGLBLID
}

#[derive(Debug, Default, Clone, Copy)]
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
            0x1200_0000 => {
                self.privileged_registers.pcrtc_mode = PcrtcMode::from(value);
                println!("Pmode = {:?}", self.privileged_registers.pcrtc_mode);
            }
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
            0x1200_00E0 => self.privileged_registers.background_color = Rgb::from(value as u32),
            0x1200_1000 => {
                let value = if value.bit(3) {
                    // VSINT
                    value & !u64::mask(3..=3)
                } else {
                    value
                };
                self.privileged_registers.status = value;
            }
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

    pub fn vblank(&mut self) {
        self.privileged_registers.status.set_bit(3, true);
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct PcrtcMode {
    pub enable_circuit1: bool,                        // EN1
    pub enable_circuit2: bool,                        // EN2
    pub alpha_value_selection: AlphaValueSelection,   // MMOD, ALP
    pub alpha_output_selection: AlphaOutputSelection, // AMOD
    pub alpha_blending_method: AlphaBlendingMethod,   // SLBG
}

impl From<u64> for PcrtcMode {
    fn from(raw: u64) -> Self {
        PcrtcMode {
            enable_circuit1: raw.bit(0),
            enable_circuit2: raw.bit(1),
            alpha_value_selection: if raw.bit(1) {
                AlphaValueSelection::Fixed(raw.bits(8..=15) as u8)
            } else {
                AlphaValueSelection::Circuit1
            },
            alpha_output_selection: AlphaOutputSelection::from_u64(raw.bits(6..=6)).unwrap(),
            alpha_blending_method: AlphaBlendingMethod::from_u64(raw.bits(7..=7)).unwrap(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum AlphaValueSelection {
    #[default]
    Circuit1,
    Fixed(u8), // ALP
}

#[derive(Debug, Default, Copy, Clone, FromPrimitive)]
pub enum AlphaOutputSelection {
    #[default]
    Circuit1 = 0,
    Circuit2 = 1,
}

#[derive(Debug, Default, Copy, Clone, FromPrimitive)]
pub enum AlphaBlendingMethod {
    #[default]
    Circuit2 = 0,
    BackgroundColor = 1,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Display {
    pub x_position: u16,              // DX
    pub y_position: u16,              // DX
    pub horizontal_magnification: u8, // MAGH
    pub vertical_magnification: u8,   // MAGV
    pub width: u16,                   // DW
    pub height: u16,                  // DH
}

impl Display {
    pub fn rect(&self) -> Rect<u16> {
        Rect {
            x_start: self.x_position,
            y_start: self.y_position,
            x_end: self.x_position + self.width,
            y_end: self.y_position + self.height,
        }
    }
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

#[derive(Debug, Default, Copy, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<u32> for Rgb {
    fn from(raw: u32) -> Self {
        Rgb {
            r: raw.bits(0..=7) as u8,
            g: raw.bits(8..=15) as u8,
            b: raw.bits(16..=23) as u8,
        }
    }
}
