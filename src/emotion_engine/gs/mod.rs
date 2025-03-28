use std::collections::VecDeque;

use enum_map::Enum;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use privileged_registers::PrivilegedRegisters;
use rendering::Vertex;

use crate::{bits::Bits, fifo::Fifo, fix::Fix};

mod privileged_registers;
mod rendering;

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pub command_queue: VecDeque<(Register, u64)>,
    privileged_registers: PrivilegedRegisters,
    registers: Registers,
    vertex_queue: Fifo<Vertex>,
}

type Fix124 = Fix<u16, 4>;

#[derive(Debug, Default)]
struct Registers {
    primitive: Primitive,                          // PRIM
    rgbaq: Rgbaq,                                  // RGBAQ
    xyz: Xyz,                                      // XYZ2
    uv: Uv,                                        // UV
    bit_blit_buffer: BitBlitBuffer,                // BITBLTBUF
    transmission_position: TransmissionPosition,   // TRXPOS
    transmission_size: TransmissionSize,           // TRXREG
    transmission_direction: TransmissionDirection, // TRXDIR
    primitive_mode_control: PrimitiveModeControl,
    transmission_pixel: u32,
    contextual: [ContextualRegisters; 2],
}

#[derive(Debug, Default)]
pub struct ContextualRegisters {
    xy_offset: XyOffset,                        // XYOFFSET_1, XYOFFSET_2
    scissor: Scissor,                           // SCISSOR_1, SCISSOR_2
    frame_buffer_settings: FrameBufferSettings, // FRAME_1, FRAME_2
    pixel_test: PixelTest,                      // TEST_1, TEST_2
    texture: Texture,                           // TEX0_1, TEX0_2, TEX2_1, TEX2_2
}

impl Gs {
    pub fn new() -> Gs {
        Gs {
            local_memory: vec![0; LOCAL_MEMORY_SIZE].into_boxed_slice(),
            command_queue: VecDeque::new(),
            privileged_registers: PrivilegedRegisters::default(),
            registers: Registers::default(),
            vertex_queue: Fifo::with_capacity(2),
        }
    }

    pub fn step(&mut self) {
        while let Some((register, data)) = self.command_queue.pop_front() {
            // println!("Command: {:?}={:x?}", register, data);
            match register {
                Register::Primitive => {
                    self.registers.primitive = Primitive::from(data);
                    self.vertex_queue.clear();
                }
                Register::Rgbaq => self.registers.rgbaq = Rgbaq::from(data),
                Register::St => todo!(),
                Register::Uv => self.registers.uv = Uv::from(data),
                Register::Xyzf2 => todo!(),
                Register::Xyz2 => {
                    self.registers.xyz = Xyz::from(data);
                    self.vertex_kick(/* drawing_kick */ true);
                }
                Register::Texture1 => self.registers.contextual[0].texture = Texture::from(data),
                Register::Texture2 => self.registers.contextual[1].texture = Texture::from(data),
                Register::Clamp1 => todo!(),
                Register::Clamp2 => todo!(),
                Register::Fog => todo!(),
                Register::Xyzf3 => todo!(),
                Register::Xyz3 => todo!(),
                Register::TextureMipMap1 => todo!(),
                Register::TextureMipMap2 => todo!(),
                Register::TextureClut1 => self.registers.contextual[0]
                    .texture
                    .update_clut_info(Texture::from(data)),
                Register::TextureClut2 => self.registers.contextual[1]
                    .texture
                    .update_clut_info(Texture::from(data)),
                Register::XyOffset1 => {
                    self.registers.contextual[0].xy_offset = XyOffset::from(data)
                }
                Register::XyOffset2 => {
                    self.registers.contextual[1].xy_offset = XyOffset::from(data)
                }
                Register::PrimitiveModeControl => {
                    self.registers.primitive_mode_control = match data.bit(0) {
                        false => PrimitiveModeControl::PrimitiveMode,
                        true => PrimitiveModeControl::Primitive,
                    }
                }
                Register::PrimitiveMode => todo!(),
                Register::TexClut => todo!(),
                Register::ScanMask => todo!(),
                Register::MipMap1_1 => todo!(),
                Register::MipMap1_2 => todo!(),
                Register::MipMap2_1 => todo!(),
                Register::MipMap2_2 => todo!(),
                Register::TextureAlpha => todo!(),
                Register::FogColor => todo!(),
                Register::TextureFlush => {}
                Register::Scissor1 => self.registers.contextual[0].scissor = Scissor::from(data),
                Register::Scissor2 => self.registers.contextual[1].scissor = Scissor::from(data),
                Register::Alpha1 => todo!(),
                Register::Alpha2 => todo!(),
                Register::DitherMatrix => todo!(),
                Register::DitherControl => todo!(),
                Register::ColorClamp => todo!(),
                Register::PixelTest1 => {
                    self.registers.contextual[0].pixel_test = PixelTest::from(data);
                    println!(
                        "Pixel test 1: {:?}",
                        self.registers.contextual[0].pixel_test
                    );
                }
                Register::PixelTest2 => {
                    self.registers.contextual[1].pixel_test = PixelTest::from(data);
                    println!(
                        "Pixel test 2: {:?}",
                        self.registers.contextual[1].pixel_test
                    );
                }
                Register::PixelAlphaBlending => todo!(),
                Register::FrameBufferAlpha1 => todo!(),
                Register::FrameBufferAlpha2 => todo!(),
                Register::FrameBuffer1 => {
                    println!("Frame buffer 1: {:x?}", data);
                    self.registers.contextual[0].frame_buffer_settings =
                        FrameBufferSettings::from(data)
                }
                Register::FrameBuffer2 => {
                    println!("Frame buffer 2: {:x?}", data);
                    self.registers.contextual[1].frame_buffer_settings =
                        FrameBufferSettings::from(data)
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
                Register::TransmissionActivation => {
                    self.registers.transmission_direction =
                        TransmissionDirection::from_u64(data.bits(0..=1)).expect("Invalid TRXDIR");
                    self.registers.transmission_pixel = 0;
                }
                Register::TransmissionData => match self.registers.transmission_direction {
                    TransmissionDirection::HostToLocal => {
                        match self.registers.transmission_position.direction {
                            PixelTransmissionOrder::UpperLeftToLowerRight => {}
                            _ => todo!(),
                        }
                        let destination_x =
                            self.registers.transmission_position.destination_x as u32;
                        let destination_y =
                            self.registers.transmission_position.destination_y as u32;
                        let width = self.registers.transmission_size.width as u32;
                        let height = self.registers.transmission_size.height as u32;
                        let pixels = width * height;
                        let buffer_width = self.registers.bit_blit_buffer.destination_width as u32;
                        let mut pixel = self.registers.transmission_pixel;
                        match self
                            .registers
                            .bit_blit_buffer
                            .destination_pixel_storage_format
                        {
                            PixelStorageFormat::Psmct32 => {
                                for i in 0..2 {
                                    let data = data.bits(i * 32..(i + 1) * 32) as u32;
                                    let x = (destination_x + pixel % width) % 2048;
                                    let y = (destination_y + pixel / width) % 2048;
                                    self.write_psmct32(
                                        self.registers.bit_blit_buffer.destination_base_pointer,
                                        x as u16,
                                        y as u16,
                                        self.registers.bit_blit_buffer.destination_width,
                                        data,
                                    );
                                    println!(
                                        "Transmitting pixel at ({x}, {y}) buffer width={buffer_width}"
                                    );
                                    pixel += 1;
                                    self.registers.transmission_pixel = pixel;
                                    if pixel == pixels {
                                        println!("Transmission of {pixels} pixels complete");
                                        self.registers.transmission_direction =
                                            TransmissionDirection::Deactivated;
                                        break;
                                    }
                                }
                            }
                            _ => todo!(),
                        }
                    }
                    TransmissionDirection::LocalToHost => todo!(),
                    TransmissionDirection::LocalToLocal => todo!(),
                    TransmissionDirection::Deactivated => todo!(),
                },
                Register::SignalSignal => todo!(),
                Register::SignalFinish => todo!(),
                Register::SignalLabel => todo!(),
            }
        }
    }

    pub fn contextual_registers(&self) -> &ContextualRegisters {
        &self.registers.contextual[self.registers.primitive.context.index()]
    }
}

#[repr(u8)]
#[derive(FromPrimitive, Debug, Enum)]
pub enum Register {
    Primitive = 0x00,              // PRIM Drawing primitive setting
    Rgbaq = 0x01,                  // RGBAQ Vertex color setting
    St = 0x02,                     // ST Vertex texture coordinate setting (texture coordinates)
    Uv = 0x03,                     // UV Vertex texture coordinate setting (texel coordinates)
    Xyzf2 = 0x04,                  // XYZF2 Vertex coordinate value setting
    Xyz2 = 0x05,                   // XYZ2 Vertex coordinate value setting
    Texture1 = 0x06,               // TEX0_1 Texture information setting
    Texture2 = 0x07,               // TEX0_2 Texture information setting
    Clamp1 = 0x08,                 // CLAMP_1 Texture wrap mode
    Clamp2 = 0x09,                 // CLAMP_2 Texture wrap mode
    Fog = 0x0a,                    // FOG Vertex fog value setting
    Xyzf3 = 0x0c,                  // XYZF3 Vertex coordinate value setting (without drawing kick)
    Xyz3 = 0x0d,                   // XYZ3 Vertex coordinate value setting (without drawing kick)
    TextureMipMap1 = 0x14,         // TEX1_1 Texture information setting
    TextureMipMap2 = 0x15,         // TEX1_2 Texture information setting
    TextureClut1 = 0x16,           // TEX2_1 Texture information setting
    TextureClut2 = 0x17,           // TEX2_2 Texture information setting
    XyOffset1 = 0x18,              // XYOFFSET_1 Offset value setting
    XyOffset2 = 0x19,              // XYOFFSET_2 Offset value setting
    PrimitiveModeControl = 0x1a,   // PRMODECONT Specification of primitive attribute setting method
    PrimitiveMode = 0x1b,          // PRMODE Drawing primitive attribute setting
    TexClut = 0x1c,                // TEXCLUT CLUT position setting
    ScanMask = 0x22,               // SCANMSK Raster address mask setting
    MipMap1_1 = 0x34,              // MIPTBP1_1 MIPMAP information setting (Level 1 ñ 3)
    MipMap1_2 = 0x35,              // MIPTBP1_2 MIPMAP information setting (Level 1 ñ 3)
    MipMap2_1 = 0x36,              // MIPTBP2_1 MIPMAP information setting (Level 4 ñ 6)
    MipMap2_2 = 0x37,              // MIPTBP2_2 MIPMAP information setting (Level 4 ñ 6)
    TextureAlpha = 0x3b,           // TEXA Texture alpha value setting
    FogColor = 0x3d,               // FOGCOL Distant fog color setting
    TextureFlush = 0x3f,           // TEXFLUSH Texture page buffer disabling
    Scissor1 = 0x40,               // SCISSOR_1 Scissoring area setting
    Scissor2 = 0x41,               // SCISSOR_2 Scissoring area setting
    Alpha1 = 0x42,                 // ALPHA_1 Alpha blending setting
    Alpha2 = 0x43,                 // ALPHA_2 Alpha blending setting
    DitherMatrix = 0x44,           // DIMX Dither matrix setting
    DitherControl = 0x45,          // DTHE Dither control
    ColorClamp = 0x46,             // COLCLAMP Color clamp control
    PixelTest1 = 0x47,             // TEST_1 Pixel test control
    PixelTest2 = 0x48,             // TEST_2 Pixel test control
    PixelAlphaBlending = 0x49,     // PABE Alpha blending control in pixel units
    FrameBufferAlpha1 = 0x4a,      // FBA_1 Alpha correction value
    FrameBufferAlpha2 = 0x4b,      // FBA_2 Alpha correction value
    FrameBuffer1 = 0x4c,           // FRAME_1 Frame buffer setting
    FrameBuffer2 = 0x4d,           // FRAME_2 Frame buffer setting
    ZBuffer1 = 0x4e,               // ZBUF_1 Z buffer setting
    ZBuffer2 = 0x4f,               // ZBUF_2 Z buffer setting
    BitBlitBuffer = 0x50,          // BITBLTBUF Setting for transmission between buffers
    TransmissionPosition = 0x51,   // TRXPOS Specification for transmission area in buffers
    TransmissionSize = 0x52,       // TRXREG Specification for transmission area in buffers
    TransmissionActivation = 0x53, // TRXDIR Activation of transmission between buffers
    TransmissionData = 0x54,       // HWREG Data port for transmission between buffers
    SignalSignal = 0x60,           // SIGNAL SIGNAL event occurrence request
    SignalFinish = 0x61,           // FINISH FINISH event occurrence request
    SignalLabel = 0x62,            // LABEL LABEL event occurrence request
}

#[derive(Debug, Clone, Copy, Default)]
struct FrameBufferSettings {
    pub base_pointer: u32,
    pub width: u16,
    pub pixel_storage_format: PixelStorageFormat,
    pub drawing_mask: u32,
}

impl From<u64> for FrameBufferSettings {
    fn from(raw: u64) -> Self {
        FrameBufferSettings {
            base_pointer: raw.bits(0..=8) as u32 * 2048,
            width: raw.bits(16..=21) as u16 * 64,
            pixel_storage_format: PixelStorageFormat::from_u64(raw.bits(24..=29))
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(24..=29))),
            drawing_mask: raw.bits(32..64) as u32,
        }
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PixelStorageFormat {
    #[default]
    Psmct32 = 0b000000,
    Psmct24 = 0b000001,
    Psmct16 = 0b000010,
    Psmct16s = 0b001010,
    Psgpu24 = 0b010010,
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
    pub x: Fix124,
    pub y: Fix124,
}

impl From<u64> for XyOffset {
    fn from(raw: u64) -> Self {
        XyOffset {
            x: Fix124::from_raw(raw.bits(0..16) as u16),
            y: Fix124::from_raw(raw.bits(32..48) as u16),
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

impl Scissor {
    pub fn contains(&self, x: u16, y: u16) -> bool {
        (self.x0..=self.x1).contains(&x) && (self.y0..=self.y1).contains(&y)
    }
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
            type_: PrimitiveType::from_u64(raw.bits(0..=2))
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

impl Context {
    pub fn index(self) -> usize {
        match self {
            Context::Context1 => 0,
            Context::Context2 => 1,
        }
    }
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
struct Xyz {
    x: Fix124,
    y: Fix124,
    z: u32,
}

impl From<u64> for Xyz {
    fn from(raw: u64) -> Self {
        Xyz {
            x: Fix124::from_raw(raw.bits(0..16) as u16),
            y: Fix124::from_raw(raw.bits(16..32) as u16),
            z: raw.bits(32..64) as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Uv {
    u: Fix124,
    v: Fix124,
}

impl From<u64> for Uv {
    fn from(raw: u64) -> Self {
        Uv {
            u: Fix124::from_raw(raw.bits(0..=13) as u16),
            v: Fix124::from_raw(raw.bits(16..=29) as u16),
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
            source_pixel_storage_format: PixelStorageFormat::from_u64(value.bits(24..=29))
                .unwrap_or_else(|| {
                    panic!("Invalid pixel storage format {:b}", value.bits(24..=29))
                }),
            destination_base_pointer: value.bits(32..=45) as u32 * 64,
            destination_width: value.bits(48..=53) as u16 * 64,
            destination_pixel_storage_format: PixelStorageFormat::from_u64(value.bits(56..=61))
                .unwrap_or_else(|| {
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

#[derive(Debug, Clone, Copy, Default)]
pub enum PrimitiveModeControl {
    #[default]
    PrimitiveMode, // Use PRMODE register
    Primitive, // Use PRIM register
}

#[derive(Debug, Clone, Copy, Default)]
struct PixelTest {
    alpha_test: AlphaTest,        // ATE + ATST
    alpha_reference: u8,          // AREF
    alpha_fail: AlphaFail,        // AFAIL
    destination_alpha_test: bool, // DATE
    destination_alpha_mode: bool, // DATM
    depth_test: DepthTest,        // ZTE + ZTST
}

impl From<u64> for PixelTest {
    fn from(raw: u64) -> Self {
        PixelTest {
            alpha_test: match raw.bit(0) {
                false => AlphaTest::Always,
                true => AlphaTest::from_u64(raw.bits(1..=3))
                    .unwrap_or_else(|| panic!("Invalid alpha test {:b}", raw.bits(1..=3))),
            },
            alpha_reference: raw.bits(4..=11) as u8,
            alpha_fail: AlphaFail::from_u64(raw.bits(12..=13))
                .unwrap_or_else(|| panic!("Invalid alpha fail {:b}", raw.bits(12..=13))),
            destination_alpha_test: raw.bit(14),
            destination_alpha_mode: raw.bit(15),
            depth_test: match raw.bit(16) {
                false => DepthTest::Always, // Not allowed according the the spec, but I'll allow it,
                true => DepthTest::from_u64(raw.bits(17..=18))
                    .unwrap_or_else(|| panic!("Invalid depth test {:b}", raw.bits(17..=18))),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum AlphaTest {
    Never = 0b000,
    #[default]
    Always = 0b001, // Same as ATE=0
    Less = 0b010,
    LessOrEqual = 0b011,
    Equal = 0b100,
    GreaterOrEqual = 0b101,
    Greater = 0b110,
    NotEqual = 0b111,
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum AlphaFail {
    #[default]
    Keep = 0b00,
    FramebufferOnly = 0b01,
    DepthBufferOnly = 0b10,
    RgbOnly = 0b11,
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum DepthTest {
    #[default]
    Never = 0b00,
    Always = 0b01,
    GreaterOrEqual = 0b10,
    Greater = 0b11,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Texture {
    base_pointer: u32,                               // TBP0
    buffer_width: u16,                               // TBW
    pixel_storage_format: PixelStorageFormat,        // PSM
    width: u16,                                      // TW
    height: u16,                                     // TH
    has_alpha: bool,                                 // TCC
    function: TextureFunction,                       // TFX
    clut_buffer_base_pointer: u32,                   // CBP
    clut_pixel_storage_format: PixelStorageFormat,   // CPSM
    clut_storage_mode: ClutStorageMode,              // CSM
    clut_entry_offset: u16,                          // CSA
    clut_buffer_load_control: ClutBufferLoadControl, // CLD
}
impl Texture {
    fn update_clut_info(&mut self, new: Texture) {
        self.pixel_storage_format = new.pixel_storage_format;
        self.clut_buffer_base_pointer = new.clut_buffer_base_pointer;
        self.clut_pixel_storage_format = new.clut_pixel_storage_format;
        self.clut_storage_mode = new.clut_storage_mode;
        self.clut_entry_offset = new.clut_entry_offset;
        self.clut_buffer_load_control = new.clut_buffer_load_control;
    }
}

impl From<u64> for Texture {
    fn from(raw: u64) -> Self {
        Texture {
            base_pointer: raw.bits(0..=13) as u32 * 64,
            buffer_width: raw.bits(14..=19) as u16 * 64,
            pixel_storage_format: PixelStorageFormat::from_u64(raw.bits(20..=25))
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(20..=25))),
            width: 2u16.pow(raw.bits(26..=29) as _),
            height: 2u16.pow(raw.bits(30..=33) as _),
            has_alpha: raw.bit(34),
            function: TextureFunction::from_u64(raw.bits(35..=36))
                .unwrap_or_else(|| panic!("Invalid texture function {:b}", raw.bits(35..=36))),
            clut_buffer_base_pointer: raw.bits(37..=50) as u32 * 64,
            clut_pixel_storage_format: PixelStorageFormat::from_u64(raw.bits(51..=54))
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(51..=54))),
            clut_storage_mode: ClutStorageMode::from_u64(raw.bits(55..=55))
                .unwrap_or_else(|| panic!("Invalid CLUT storage mode {:b}", raw.bits(55..=55))),
            clut_entry_offset: raw.bits(56..=60) as u16 * 16,
            clut_buffer_load_control: ClutBufferLoadControl::from_u64(raw.bits(61..=63))
                .unwrap_or_else(|| {
                    panic!("Invalid CLUT buffer load control {:b}", raw.bits(61..=63))
                }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum TextureFunction {
    #[default]
    Modulate = 0b00,
    Decal = 0b01,
    Highlight = 0b10,
    Highlight2 = 0b11,
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum ClutStorageMode {
    #[default]
    Csm1,
    Csm2,
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum ClutBufferLoadControl {
    #[default]
    NotChanged = 0b000,
    LoadFromCsa = 0b001,
    LoadFromCsaCopyToCbp0 = 0b010,
    LoadFromCsaCopyToCbp1 = 0b011,
    LoadFromCbpCopyToCbp0 = 0b100,
    LoadFromCbpCopyToCbp1 = 0b101,
}
