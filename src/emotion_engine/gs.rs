use std::collections::VecDeque;

use enum_map::{Enum, EnumMap};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes};

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pub command_queue: VecDeque<(Register, u64)>,
    pcrtc_mode: u64,                                 // PMODE
    sync_mode1: u64,                                 // SMODE1
    sync_mode2: u64,                                 // SMODE2
    dram_refresh: u64,                               // SRFSH
    synch1: u64,                                     // SYNCH1
    synch2: u64,                                     // SYNCH2
    syncv: u64,                                      // SYNCV
    display_frame_buffer1: u64,                      // DISPFB1
    display1: u64,                                   // DISPLAY1
    display_frame_buffer2: u64,                      // DISPFB1
    display2: u64,                                   // DISPLAY1
    write_buffer: u64,                               // EXTBUF
    write_data: u64,                                 // EXTDATA
    write_start: u64,                                // EXTWRITE
    background_color: u64,                           // BGCOLOR
    status: u64,                                     // CSR
    interrupt_mask: u64,                             // IMR
    bus_direction: u64,                              // BUSDIR
    signal_label_id: u64,                            // SIGLBLID
    primitive: Primitive,                            // PRIM
    rgbaq: Rgbaq,                                    // RGBAQ
    frame_buffer_settings: [FrameBufferSettings; 2], // FRAME_1, FRAME_2
    xy_offset: [XyOffset; 2],                        // XYOFFSET_1, XYOFFSET_2
    scissor: [Scissor; 2],                           // SCISSOR_1, SCISSOR_2
}

impl Gs {
    pub fn new() -> Gs {
        Gs {
            local_memory: vec![0; LOCAL_MEMORY_SIZE].into_boxed_slice(),
            command_queue: VecDeque::new(),
            pcrtc_mode: 0,
            sync_mode1: 0,
            sync_mode2: 0,
            dram_refresh: 0,
            synch1: 0,
            synch2: 0,
            syncv: 0,
            display_frame_buffer1: 0,
            display1: 0,
            display_frame_buffer2: 0,
            display2: 0,
            write_buffer: 0,
            write_data: 0,
            write_start: 0,
            background_color: 0,
            status: 0,
            interrupt_mask: 0,
            bus_direction: 0,
            signal_label_id: 0,
            primitive: Primitive::from(0),
            rgbaq: Rgbaq::from(0),
            frame_buffer_settings: [FrameBufferSettings::from(0); 2],
            xy_offset: [XyOffset::from(0); 2],
            scissor: [Scissor::from(0); 2],
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
            0x1200_0000 => self.pcrtc_mode = value,
            0x1200_0010 => self.sync_mode1 = value,
            0x1200_0020 => self.sync_mode2 = value,
            0x1200_0030 => self.dram_refresh = value,
            0x1200_0040 => self.synch1 = value,
            0x1200_0050 => self.synch2 = value,
            0x1200_0060 => self.syncv = value,
            0x1200_0070 => self.display_frame_buffer1 = value,
            0x1200_0080 => self.display1 = value,
            0x1200_0090 => self.display_frame_buffer2 = value,
            0x1200_00A0 => self.display2 = value,
            0x1200_00B0 => self.write_buffer = value,
            0x1200_00C0 => self.write_data = value,
            0x1200_00D0 => self.write_start = value,
            0x1200_00E0 => self.background_color = value,
            0x1200_1000 => self.status = value,
            0x1200_1010 => self.interrupt_mask = value,
            0x1200_1040 => self.bus_direction = value,
            0x1200_1080 => self.signal_label_id = value,
            _ => panic!("Invalid GS write64 {} to address: 0x{:08x}", value, address),
        }
    }

    pub fn read_privileged64(&self, address: u32) -> u64 {
        match address {
            0x1200_0000 => self.pcrtc_mode,
            0x1200_0010 => self.sync_mode1,
            0x1200_0020 => self.sync_mode2,
            0x1200_0030 => self.dram_refresh,
            0x1200_0040 => self.synch1,
            0x1200_0050 => self.synch2,
            0x1200_0060 => self.syncv,
            0x1200_0070 => self.display_frame_buffer1,
            0x1200_0080 => self.display1,
            0x1200_0090 => self.display_frame_buffer2,
            0x1200_00A0 => self.display2,
            0x1200_00B0 => self.write_buffer,
            0x1200_00C0 => self.write_data,
            0x1200_00D0 => self.write_start,
            0x1200_00E0 => self.background_color,
            0x1200_1000 => self.status,
            0x1200_1010 => self.interrupt_mask,
            0x1200_1040 => self.bus_direction,
            0x1200_1080 => self.signal_label_id,
            _ => panic!("Invalid GS read64 from address: 0x{:08x}", address),
        }
    }

    pub fn step(&mut self) {
        while let Some((register, data)) = self.command_queue.pop_front() {
            println!("Command: {:?}={:x?}", register, data);
            match register {
                Register::Primitive => self.primitive = Primitive::from(data),
                Register::Rgbaq => self.rgbaq = Rgbaq::from(data),
                Register::St => todo!(),
                Register::Uv => todo!(),
                Register::Xyzf2 => todo!(),
                Register::Xyz2 => todo!(),
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
                Register::XyOffset1 => self.xy_offset[0] = XyOffset::from(data),
                Register::XyOffset2 => self.xy_offset[1] = XyOffset::from(data),
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
                Register::Scissor1 => self.scissor[0] = Scissor::from(data),
                Register::Scissor2 => self.scissor[1] = Scissor::from(data),
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
                    self.frame_buffer_settings[0] = FrameBufferSettings::from(data)
                }
                Register::FrameBuffer2 => {
                    self.frame_buffer_settings[1] = FrameBufferSettings::from(data)
                }
                Register::ZBuffer1 => todo!(),
                Register::ZBuffer2 => todo!(),
                Register::BitBlitBuffer => todo!(),
                Register::TransmissionPosition => todo!(),
                Register::TransmissionSize => todo!(),
                Register::TransmissionDirection => todo!(),
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

#[derive(Debug, Clone, Copy)]
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

#[repr(u8)]
#[derive(FromPrimitive, Debug, Clone, Copy)]
enum PixelStorageFormat {
    Psmct32 = 0b000000,
    Psmct24 = 0b000001,
    Psmct16 = 0b000010,
    Psmct16s = 0b001010,
    Psmz32 = 0b110000,
    Psmz24 = 0b110001,
    Psmz16 = 0b110010,
    Psmz16s = 0b111010,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[repr(u8)]
#[derive(FromPrimitive, Debug, Clone, Copy)]
enum PrimitiveType {
    Point,
    Line,
    LineStrip,
    Triangle,
    TriangleStrip,
    TriangleFan,
    Sprite,
    SpecificationProhibited,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum ShadingMethod {
    Flat,
    Gouraud,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum TextureCoordinateMethod {
    Stq,
    Uv,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Context {
    Context1,
    Context2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum FragmentValueControl {
    Unfixed,
    Fixed,
}

#[derive(Debug, Clone, Copy)]
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
