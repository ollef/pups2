use std::collections::VecDeque;

use enum_map::Enum;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes};

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pub command_queue: VecDeque<(Register, u64)>,
    privileged_registers: PrivilegedRegisters,
    registers: Registers,
}

#[derive(Debug, Default)]
struct PrivilegedRegisters {
    pcrtc_mode: u64,            // PMODE
    sync_mode1: u64,            // SMODE1
    sync_mode2: u64,            // SMODE2
    dram_refresh: u64,          // SRFSH
    synch1: u64,                // SYNCH1
    synch2: u64,                // SYNCH2
    syncv: u64,                 // SYNCV
    display_frame_buffer1: u64, // DISPFB1
    display1: u64,              // DISPLAY1
    display_frame_buffer2: u64, // DISPFB1
    display2: u64,              // DISPLAY1
    write_buffer: u64,          // EXTBUF
    write_data: u64,            // EXTDATA
    write_start: u64,           // EXTWRITE
    background_color: u64,      // BGCOLOR
    status: u64,                // CSR
    interrupt_mask: u64,        // IMR
    bus_direction: u64,         // BUSDIR
    signal_label_id: u64,       // SIGLBLID
}

#[derive(Debug, Default)]
struct Registers {
    primitive: Primitive,                            // PRIM
    rgbaq: Rgbaq,                                    // RGBAQ
    xyz2: Xyz2,                                      // XYZ2
    xy_offset: [XyOffset; 2],                        // XYOFFSET_1, XYOFFSET_2
    scissor: [Scissor; 2],                           // SCISSOR_1, SCISSOR_2
    frame_buffer_settings: [FrameBufferSettings; 2], // FRAME_1, FRAME_2
    bit_blit_buffer: BitBlitBuffer,                  // BITBLTBUF
    transmission_position: TransmissionPosition,     // TRXPOS
    transmission_size: TransmissionSize,             // TRXREG
    transmission_direction: TransmissionDirection,   // TRXDIR
}

impl Gs {
    pub fn new() -> Gs {
        Gs {
            local_memory: vec![0; LOCAL_MEMORY_SIZE].into_boxed_slice(),
            command_queue: VecDeque::new(),
            privileged_registers: PrivilegedRegisters::default(),
            registers: Registers::default(),
        }
    }

    pub fn write_privileged<T: Bytes>(&mut self, address: u32, value: T) {
        match std::mem::size_of::<T>() {
            8 => self.write_privileged64(address, u64::from_bytes(value.to_bytes().as_ref())),
            _ => panic!("Invalid GS write size: {}", std::mem::size_of::<T>()),
        }
    }
    pub fn read_privileged<T: Bytes>(&self, address: u32) -> T {
        match std::mem::size_of::<T>() {
            8 => T::from_bytes(self.read_privileged64(address).to_bytes().as_ref()),
            _ => panic!("Invalid GS read size: {}", std::mem::size_of::<T>()),
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
            0x1200_0070 => self.privileged_registers.display_frame_buffer1 = value,
            0x1200_0080 => self.privileged_registers.display1 = value,
            0x1200_0090 => self.privileged_registers.display_frame_buffer2 = value,
            0x1200_00A0 => self.privileged_registers.display2 = value,
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
            0x1200_0000 => self.privileged_registers.pcrtc_mode,
            0x1200_0010 => self.privileged_registers.sync_mode1,
            0x1200_0020 => self.privileged_registers.sync_mode2,
            0x1200_0030 => self.privileged_registers.dram_refresh,
            0x1200_0040 => self.privileged_registers.synch1,
            0x1200_0050 => self.privileged_registers.synch2,
            0x1200_0060 => self.privileged_registers.syncv,
            0x1200_0070 => self.privileged_registers.display_frame_buffer1,
            0x1200_0080 => self.privileged_registers.display1,
            0x1200_0090 => self.privileged_registers.display_frame_buffer2,
            0x1200_00A0 => self.privileged_registers.display2,
            0x1200_00B0 => self.privileged_registers.write_buffer,
            0x1200_00C0 => self.privileged_registers.write_data,
            0x1200_00D0 => self.privileged_registers.write_start,
            0x1200_00E0 => self.privileged_registers.background_color,
            0x1200_1000 => self.privileged_registers.status,
            0x1200_1010 => self.privileged_registers.interrupt_mask,
            0x1200_1040 => self.privileged_registers.bus_direction,
            0x1200_1080 => self.privileged_registers.signal_label_id,
            _ => panic!("Invalid GS read64 from address: 0x{:08x}", address),
        }
    }

    pub fn step(&mut self) {
        while let Some((register, data)) = self.command_queue.pop_front() {
            println!("Command: {:?}={:x?}", register, data);
            match register {
                Register::Primitive => self.registers.primitive = Primitive::from(data),
                Register::Rgbaq => self.registers.rgbaq = Rgbaq::from(data),
                Register::St => todo!(),
                Register::Uv => todo!(),
                Register::Xyzf2 => todo!(),
                Register::Xyz2 => self.registers.xyz2 = Xyz2::from(data),
                Register::Tex0_1 => todo!(),
                Register::Tex0_2 => todo!(),
                Register::Clamp1 => todo!(),
                Register::Clamp2 => todo!(),
                Register::Fog => todo!(),
                Register::Xyzf3 => todo!(),
                Register::Xyz3 => todo!(),
                Register::Texture1_1 => todo!(),
                Register::Texture1_2 => todo!(),
                Register::Texture2_1 => todo!(),
                Register::Texture2_2 => todo!(),
                Register::XyOffset1 => self.registers.xy_offset[0] = XyOffset::from(data),
                Register::XyOffset2 => self.registers.xy_offset[1] = XyOffset::from(data),
                Register::PrimitiveModeControl => todo!(),
                Register::PrimitiveMode => todo!(),
                Register::TexClut => todo!(),
                Register::ScanMask => todo!(),
                Register::MipMap1_1 => todo!(),
                Register::MipMap1_2 => todo!(),
                Register::MipMap2_1 => todo!(),
                Register::MipMap2_2 => todo!(),
                Register::TextureAlpha => todo!(),
                Register::FogColor => todo!(),
                Register::TextureFlush => todo!(),
                Register::Scissor1 => self.registers.scissor[0] = Scissor::from(data),
                Register::Scissor2 => self.registers.scissor[1] = Scissor::from(data),
                Register::Alpha1 => todo!(),
                Register::Alpha2 => todo!(),
                Register::DitherMatrix => todo!(),
                Register::DitherControl => todo!(),
                Register::ColorClamp => todo!(),
                Register::PixelTest1 => todo!(),
                Register::PixelTest2 => todo!(),
                Register::PixelAlphaBlending => todo!(),
                Register::FrameBufferAlpha1 => todo!(),
                Register::FrameBufferAlpha2 => todo!(),
                Register::FrameBuffer1 => {
                    self.registers.frame_buffer_settings[0] = FrameBufferSettings::from(data)
                }
                Register::FrameBuffer2 => {
                    self.registers.frame_buffer_settings[1] = FrameBufferSettings::from(data)
                }
                Register::ZBuffer1 => todo!(),
                Register::ZBuffer2 => todo!(),
                Register::BitBlitBuffer => {
                    self.registers.bit_blit_buffer = BitBlitBuffer::from(data)
                }
                Register::TransmissionPosition => {
                    self.registers.transmission_position = TransmissionPosition::from(data)
                }
                Register::TransmissionSize => {
                    self.registers.transmission_size = TransmissionSize::from(data)
                }
                Register::TransmissionDirection => {
                    self.registers.transmission_direction =
                        TransmissionDirection::from_u64(data.bits(0..=1)).expect("Invalid TRXDIR")
                }
                Register::TransmissionData => todo!(),
                Register::SignalSignal => todo!(),
                Register::SignalFinish => todo!(),
                Register::SignalLabel => todo!(),
            }
        }
    }
}

#[repr(u8)]
#[derive(FromPrimitive, Debug, Enum)]
pub enum Register {
    Primitive = 0x00,             // PRIM Drawing primitive setting
    Rgbaq = 0x01,                 // RGBAQ Vertex color setting
    St = 0x02,                    // ST Vertex texture coordinate setting (texture coordinates)
    Uv = 0x03,                    // UV Vertex texture coordinate setting (texel coordinates)
    Xyzf2 = 0x04,                 // XYZF2 Vertex coordinate value setting
    Xyz2 = 0x05,                  // XYZ2 Vertex coordinate value setting
    Tex0_1 = 0x06,                // TEX0_1 Texture information setting
    Tex0_2 = 0x07,                // TEX0_2 Texture information setting
    Clamp1 = 0x08,                // CLAMP_1 Texture wrap mode
    Clamp2 = 0x09,                // CLAMP_2 Texture wrap mode
    Fog = 0x0a,                   // FOG Vertex fog value setting
    Xyzf3 = 0x0c,                 // XYZF3 Vertex coordinate value setting (without drawing kick)
    Xyz3 = 0x0d,                  // XYZ3 Vertex coordinate value setting (without drawing kick)
    Texture1_1 = 0x14,            // TEX1_1 Texture information setting
    Texture1_2 = 0x15,            // TEX1_2 Texture information setting
    Texture2_1 = 0x16,            // TEX2_1 Texture information setting
    Texture2_2 = 0x17,            // TEX2_2 Texture information setting
    XyOffset1 = 0x18,             // XYOFFSET_1 Offset value setting
    XyOffset2 = 0x19,             // XYOFFSET_2 Offset value setting
    PrimitiveModeControl = 0x1a,  // PRMODECONT Specification of primitive attribute setting method
    PrimitiveMode = 0x1b,         // PRMODE Drawing primitive attribute setting
    TexClut = 0x1c,               // TEXCLUT CLUT position setting
    ScanMask = 0x22,              // SCANMSK Raster address mask setting
    MipMap1_1 = 0x34,             // MIPTBP1_1 MIPMAP information setting (Level 1 ñ 3)
    MipMap1_2 = 0x35,             // MIPTBP1_2 MIPMAP information setting (Level 1 ñ 3)
    MipMap2_1 = 0x36,             // MIPTBP2_1 MIPMAP information setting (Level 4 ñ 6)
    MipMap2_2 = 0x37,             // MIPTBP2_2 MIPMAP information setting (Level 4 ñ 6)
    TextureAlpha = 0x3b,          // TEXA Texture alpha value setting
    FogColor = 0x3d,              // FOGCOL Distant fog color setting
    TextureFlush = 0x3f,          // TEXFLUSH Texture page buffer disabling
    Scissor1 = 0x40,              // SCISSOR_1 Scissoring area setting
    Scissor2 = 0x41,              // SCISSOR_2 Scissoring area setting
    Alpha1 = 0x42,                // ALPHA_1 Alpha blending setting
    Alpha2 = 0x43,                // ALPHA_2 Alpha blending setting
    DitherMatrix = 0x44,          // DIMX Dither matrix setting
    DitherControl = 0x45,         // DTHE Dither control
    ColorClamp = 0x46,            // COLCLAMP Color clamp control
    PixelTest1 = 0x47,            // TEST_1 Pixel test control
    PixelTest2 = 0x48,            // TEST_2 Pixel test control
    PixelAlphaBlending = 0x49,    // PABE Alpha blending control in pixel units
    FrameBufferAlpha1 = 0x4a,     // FBA_1 Alpha correction value
    FrameBufferAlpha2 = 0x4b,     // FBA_2 Alpha correction value
    FrameBuffer1 = 0x4c,          // FRAME_1 Frame buffer setting
    FrameBuffer2 = 0x4d,          // FRAME_2 Frame buffer setting
    ZBuffer1 = 0x4e,              // ZBUF_1 Z buffer setting
    ZBuffer2 = 0x4f,              // ZBUF_2 Z buffer setting
    BitBlitBuffer = 0x50,         // BITBLTBUF Setting for transmission between buffers
    TransmissionPosition = 0x51,  // TRXPOS Specification for transmission area in buffers
    TransmissionSize = 0x52,      // TRXREG Specification for transmission area in buffers
    TransmissionDirection = 0x53, // TRXDIR Activation of transmission between buffers
    TransmissionData = 0x54,      // HWREG Data port for transmission between buffers
    SignalSignal = 0x60,          // SIGNAL SIGNAL event occurrence request
    SignalFinish = 0x61,          // FINISH FINISH event occurrence request
    SignalLabel = 0x62,           // LABEL LABEL event occurrence request
}

#[derive(Debug, Clone, Copy, Default)]
struct FrameBufferSettings {
    pub base_pointer: u32,
    pub width: u32,
    pub pixel_storage_format: PixelStorageFormat,
    pub drawing_mask: u32,
}

impl From<u64> for FrameBufferSettings {
    fn from(raw: u64) -> Self {
        FrameBufferSettings {
            base_pointer: raw.bits(0..=8) as u32 * 2048,
            width: raw.bits(16..=21) as u32 * 64,
            pixel_storage_format: PixelStorageFormat::from_u8(raw.bits(24..=29) as u8)
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(24..=29))),
            drawing_mask: raw.bits(32..64) as u32,
        }
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default)]
enum PixelStorageFormat {
    #[default]
    Psmct32 = 0b000000,
    Psmct24 = 0b000001,
    Psmct16 = 0b000010,
    Psmct16s = 0b001010,
    Psmt8 = 0b010011,
    Psmt4 = 0b010100,
    Psmt8h = 0b011011,
    Psmt4hl = 0b100100,
    Psmt4hh = 0b101100,
    Psmz32 = 0b110000,
    Psmz24 = 0b110001,
    Psmz16 = 0b110010,
    Psmz16s = 0b111010,
}

#[derive(Debug, Clone, Copy, Default)]
struct XyOffset {
    pub x: u16,
    pub y: u16,
}

impl From<u64> for XyOffset {
    fn from(raw: u64) -> Self {
        XyOffset {
            x: raw.bits(0..16) as u16,
            y: raw.bits(32..48) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Scissor {
    pub x0: u16,
    pub x1: u16,
    pub y0: u16,
    pub y1: u16,
}

impl From<u64> for Scissor {
    fn from(raw: u64) -> Self {
        Scissor {
            x0: raw.bits(0..=10) as u16,
            x1: raw.bits(16..=26) as u16,
            y0: raw.bits(32..=42) as u16,
            y1: raw.bits(48..=58) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Primitive {
    type_: PrimitiveType,                               // PRIM
    shading_method: ShadingMethod,                      // IIP
    texture_mapping: bool,                              // TME
    fogging: bool,                                      // FGE
    alpha_blending: bool,                               // ABE
    anti_aliasing: bool,                                // AA1
    texture_coordinate_method: TextureCoordinateMethod, // FST
    context: Context,                                   // CTXT
    fragment_value_control: FragmentValueControl,       // FIX
}

impl From<u64> for Primitive {
    fn from(raw: u64) -> Self {
        Primitive {
            type_: PrimitiveType::from_u8(raw.bits(0..=2) as u8)
                .unwrap_or_else(|| panic!("Invalid primitive type {:b}", raw.bits(0..=2))),
            shading_method: match raw.bit(3) {
                false => ShadingMethod::Flat,
                true => ShadingMethod::Gouraud,
            },
            texture_mapping: raw.bit(4),
            fogging: raw.bit(5),
            alpha_blending: raw.bit(6),
            anti_aliasing: raw.bit(7),
            texture_coordinate_method: match raw.bit(8) {
                false => TextureCoordinateMethod::Stq,
                true => TextureCoordinateMethod::Uv,
            },
            context: match raw.bit(9) {
                false => Context::Context1,
                true => Context::Context2,
            },
            fragment_value_control: match raw.bit(10) {
                false => FragmentValueControl::Unfixed,
                true => FragmentValueControl::Fixed,
            },
        }
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default)]
enum PrimitiveType {
    #[default]
    Point,
    Line,
    LineStrip,
    Triangle,
    TriangleStrip,
    TriangleFan,
    Sprite,
    SpecificationProhibited,
}

#[derive(Debug, Clone, Copy, Default)]
enum ShadingMethod {
    #[default]
    Flat,
    Gouraud,
}

#[derive(Debug, Clone, Copy, Default)]
enum TextureCoordinateMethod {
    #[default]
    Stq,
    Uv,
}

#[derive(Debug, Clone, Copy, Default)]
enum Context {
    #[default]
    Context1,
    Context2,
}

#[derive(Debug, Clone, Copy, Default)]
enum FragmentValueControl {
    #[default]
    Unfixed,
    Fixed,
}

#[derive(Debug, Clone, Copy, Default)]
struct Rgbaq {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    q: f32,
}

impl From<u64> for Rgbaq {
    fn from(raw: u64) -> Self {
        Rgbaq {
            r: raw.bits(0..8) as u8,
            g: raw.bits(8..16) as u8,
            b: raw.bits(16..24) as u8,
            a: raw.bits(24..32) as u8,
            q: f32::from_bits(raw.bits(32..64) as u32),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Xyz2 {
    x: u16,
    y: u16,
    z: u32,
}

impl From<u64> for Xyz2 {
    fn from(raw: u64) -> Self {
        Xyz2 {
            x: raw.bits(0..16) as u16,
            y: raw.bits(16..32) as u16,
            z: raw.bits(32..64) as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct BitBlitBuffer {
    source_base_pointer: u32,
    source_width: u16,
    source_pixel_storage_format: PixelStorageFormat,
    destination_base_pointer: u32,
    destination_width: u16,
    destination_pixel_storage_format: PixelStorageFormat,
}

impl From<u64> for BitBlitBuffer {
    fn from(value: u64) -> Self {
        BitBlitBuffer {
            source_base_pointer: value.bits(0..=13) as u32 * 64,
            source_width: value.bits(16..=21) as u16 * 64,
            source_pixel_storage_format: PixelStorageFormat::from_u8(value.bits(24..=29) as u8)
                .unwrap_or_else(|| {
                    panic!("Invalid pixel storage format {:b}", value.bits(24..=29))
                }),
            destination_base_pointer: value.bits(32..=45) as u32 * 64,
            destination_width: value.bits(48..=53) as u16 * 64,
            destination_pixel_storage_format:
                PixelStorageFormat::from_u8(value.bits(56..=61) as u8).unwrap_or_else(|| {
                    panic!("Invalid pixel storage format {:b}", value.bits(56..=61))
                }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TransmissionPosition {
    source_x: u16,
    source_y: u16,
    destination_x: u16,
    destination_y: u16,
    direction: PixelTransmissionOrder,
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
enum PixelTransmissionOrder {
    #[default]
    UpperLeftToLowerRight,
    LowerLeftToUpperRight,
    UpperRightToLowerLeft,
    LowerRightToUpperLeft,
}

impl From<u64> for TransmissionPosition {
    fn from(raw: u64) -> Self {
        TransmissionPosition {
            source_x: raw.bits(0..=10) as u16,
            source_y: raw.bits(16..=26) as u16,
            destination_x: raw.bits(32..=42) as u16,
            destination_y: raw.bits(48..=58) as u16,
            direction: PixelTransmissionOrder::from_u64(raw.bits(59..=60)).unwrap_or_else(|| {
                panic!("Invalid pixel transmission order {:b}", raw.bits(59..=60))
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TransmissionSize {
    width: u16,
    height: u16,
}

impl From<u64> for TransmissionSize {
    fn from(raw: u64) -> Self {
        TransmissionSize {
            width: raw.bits(0..=11) as u16,
            height: raw.bits(32..=43) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum TransmissionDirection {
    #[default]
    HostToLocal,
    LocalToHost,
    LocalToLocal,
    Deactivated,
}
