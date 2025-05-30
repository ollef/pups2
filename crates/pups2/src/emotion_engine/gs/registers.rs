use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes, fix::Fix};

use super::Gs;

type Fix124 = Fix<u16, 4>;

#[derive(Debug, Default)]
pub struct Registers {
    pub primitive: Primitive,                          // PRIM
    pub rgbaq: Rgbaq,                                  // RGBAQ
    pub xyz: Xyz,                                      // XYZ2
    pub uv: Uv,                                        // UV
    pub fog: u8,                                       // FOG
    pub bit_blit_buffer: BitBlitBuffer,                // BITBLTBUF
    pub transmission_position: TransmissionPosition,   // TRXPOS
    pub transmission_size: TransmissionSize,           // TRXREG
    pub transmission_direction: TransmissionDirection, // TRXDIR
    pub dither_control: DitherControl,                 // DTHE
    pub color_clamp: ColorClamp,                       // COLCLAMP
    pub primitive_mode_control: PrimitiveModeControl,
    pub transmission_pixel: u32,
    pub contextual: [ContextualRegisters; 2],
}

#[derive(Debug, Default)]
pub struct ContextualRegisters {
    pub xy_offset: XyOffset,                        // XYOFFSET_1, XYOFFSET_2
    pub scissor: Scissor,                           // SCISSOR_1, SCISSOR_2
    pub frame_buffer_settings: FrameBufferSettings, // FRAME_1, FRAME_2
    pub pixel_test: PixelTest,                      // TEST_1, TEST_2
    pub texture: Texture,                           // TEX0_1, TEX0_2, TEX2_1, TEX2_2
    pub z_buffer_settings: ZBufferSettings,         // ZBUF_1, ZBUF_2
    pub alpha: Alpha,                               // ALPHA_1, ALPHA_2
}

#[derive(FromPrimitive, Debug, Clone, Copy)]
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

impl Gs {
    pub fn contextual_registers(&self) -> &ContextualRegisters {
        &self.registers.contextual[self.registers.primitive.context.index()]
    }

    pub fn write_register(&mut self, register: Register, data: u64) {
        match register {
            Register::Primitive => {
                self.registers.primitive = Primitive::from(data);
                self.vertex_queue.clear();
            }
            Register::Rgbaq => self.registers.rgbaq = Rgbaq::from(data),
            Register::St => todo!(),
            Register::Uv => self.registers.uv = Uv::from(data),
            Register::Xyzf2 => {
                self.registers.xyz = Xyz {
                    x: Fix124::from_raw(data.bits(0..16) as u16),
                    y: Fix124::from_raw(data.bits(16..32) as u16),
                    z: data.bits(32..=55) as u32,
                };
                self.registers.fog = data.bits(56..=63) as u8;
                self.vertex_kick(/* drawing_kick */ true);
            }
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
            Register::XyOffset1 => self.registers.contextual[0].xy_offset = XyOffset::from(data),
            Register::XyOffset2 => self.registers.contextual[1].xy_offset = XyOffset::from(data),
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
            Register::Alpha1 => self.registers.contextual[0].alpha = Alpha::from(data),
            Register::Alpha2 => self.registers.contextual[1].alpha = Alpha::from(data),
            Register::DitherMatrix => todo!(),
            Register::DitherControl => self.registers.dither_control = DitherControl::from(data),
            Register::ColorClamp => self.registers.color_clamp = ColorClamp::from(data),
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
                self.registers.contextual[0].frame_buffer_settings =
                    FrameBufferSettings::from(data);
                println!(
                    "Frame buffer 1: {:x?}",
                    self.registers.contextual[0].frame_buffer_settings
                );
            }
            Register::FrameBuffer2 => {
                self.registers.contextual[1].frame_buffer_settings =
                    FrameBufferSettings::from(data);
                println!(
                    "Frame buffer 2: {:x?}",
                    self.registers.contextual[1].frame_buffer_settings
                );
            }
            Register::ZBuffer1 => {
                self.registers.contextual[0].z_buffer_settings = ZBufferSettings::from(data);
                println!(
                    "Z buffer 1: {:x?}",
                    self.registers.contextual[0].z_buffer_settings
                );
            }
            Register::ZBuffer2 => {
                self.registers.contextual[1].z_buffer_settings = ZBufferSettings::from(data);
                println!(
                    "Z buffer 2: {:x?}",
                    self.registers.contextual[1].z_buffer_settings
                );
            }
            Register::BitBlitBuffer => self.registers.bit_blit_buffer = BitBlitBuffer::from(data),
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
                match self.registers.transmission_direction {
                    TransmissionDirection::HostToLocal => {}
                    TransmissionDirection::LocalToHost => {
                        todo!()
                    }
                    TransmissionDirection::LocalToLocal => {
                        match self.registers.transmission_position.order {
                            PixelTransmissionOrder::UpperLeftToLowerRight => {}
                            _ => todo!(),
                        }
                        let source_x = self.registers.transmission_position.source_x as u32;
                        let source_y = self.registers.transmission_position.source_y as u32;
                        let destination_x =
                            self.registers.transmission_position.destination_x as u32;
                        let destination_y =
                            self.registers.transmission_position.destination_y as u32;
                        let width = self.registers.transmission_size.width as u32;
                        let height = self.registers.transmission_size.height as u32;
                        println!(
                            "Local transmission of {width}x{height} pixels from ({source_x}, {source_y}) to ({destination_x}, {destination_y})"
                        );
                        println!(
                            "Source width: {}",
                            self.registers.bit_blit_buffer.source_width
                        );
                        println!(
                            "Destination width: {}",
                            self.registers.bit_blit_buffer.destination_width
                        );
                        println!(
                            "Source base pointer: {:x?}",
                            self.registers.bit_blit_buffer.source_base_pointer
                        );
                        println!(
                            "Destination base pointer: {:x?}",
                            self.registers.bit_blit_buffer.destination_base_pointer
                        );
                        let pixels = width * height;
                        self.tmp_data.reserve(pixels as usize * 4);

                        for pixel in 0..pixels {
                            let x = (source_x + pixel % width) % 2048;
                            let y = (source_y + pixel / width) % 2048;
                            match self.registers.bit_blit_buffer.source_pixel_storage_format {
                                PixelStorageFormat::Ct32 => {
                                    let data = self.read_psmct32(
                                        self.registers.bit_blit_buffer.source_base_pointer,
                                        x as u16,
                                        y as u16,
                                        self.registers.bit_blit_buffer.source_width,
                                    );
                                    self.tmp_data.extend_from_slice(&data.to_bytes());
                                }
                                _ => todo!(),
                            }
                        }
                        for pixel in 0..pixels {
                            let x = (destination_x + pixel % width) % 2048;
                            let y = (destination_y + pixel / width) % 2048;
                            match self
                                .registers
                                .bit_blit_buffer
                                .destination_pixel_storage_format
                            {
                                PixelStorageFormat::Ct32 => {
                                    let data = u32::from_bytes(
                                        &self.tmp_data
                                            [pixel as usize * 4..(pixel + 1) as usize * 4],
                                    );
                                    self.write_psmct32(
                                        self.registers.bit_blit_buffer.destination_base_pointer,
                                        x as u16,
                                        y as u16,
                                        self.registers.bit_blit_buffer.destination_width,
                                        data,
                                    );
                                }
                                _ => todo!(),
                            }
                        }
                        self.tmp_data.clear();

                        self.registers.transmission_direction = TransmissionDirection::Deactivated;
                    }
                    TransmissionDirection::Deactivated => {}
                }
            }
            Register::TransmissionData => match self.registers.transmission_direction {
                TransmissionDirection::HostToLocal => {
                    let destination_x = self.registers.transmission_position.destination_x as u32;
                    let destination_y = self.registers.transmission_position.destination_y as u32;
                    let width = self.registers.transmission_size.width as u32;
                    let height = self.registers.transmission_size.height as u32;
                    let pixels = width * height;
                    let mut pixel = self.registers.transmission_pixel;
                    match self
                        .registers
                        .bit_blit_buffer
                        .destination_pixel_storage_format
                    {
                        PixelStorageFormat::Ct32 => {
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
                                // println!(
                                //     "Transmitting pixel at ({x}, {y}) buffer width={buffer_width}"
                                // );
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
                TransmissionDirection::LocalToLocal => panic!("Can't happen"),
                TransmissionDirection::Deactivated => todo!(),
            },
            Register::SignalSignal => todo!(),
            Register::SignalFinish => todo!(),
            Register::SignalLabel => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameBufferSettings {
    pub base_pointer: u32,
    pub width: u16,
    pub pixel_storage_format: PixelStorageFormat,
    pub drawing_mask: u32,
}

impl From<u64> for FrameBufferSettings {
    fn from(raw: u64) -> Self {
        FrameBufferSettings {
            base_pointer: raw.bits(0..=8) as u32 * 2048 * 4,
            width: raw.bits(16..=21) as u16 * 64,
            pixel_storage_format: PixelStorageFormat::from_u64(raw.bits(24..=29))
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(24..=29))),
            drawing_mask: raw.bits(32..64) as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ZBufferSettings {
    pub base_pointer: u32,              // ZBP
    pub storage_format: ZStorageFormat, // PSM
    pub no_update: bool,                // ZMSK
}

impl From<u64> for ZBufferSettings {
    fn from(raw: u64) -> Self {
        ZBufferSettings {
            base_pointer: raw.bits(0..=8) as u32 * 2048 * 4,
            storage_format: ZStorageFormat::from_u64(raw.bits(24..=27))
                .unwrap_or_else(|| panic!("Invalid Z storage format {:b}", raw.bits(24..=27))),
            no_update: raw.bit(32),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Alpha {
    pub input_color_a: InputColor, // A
    pub input_color_b: InputColor, // B
    pub input_alpha_c: InputAlpha, // C
    pub input_color_d: InputColor, // D
    pub fixed: u8,                 // FIX
}

impl From<u64> for Alpha {
    fn from(raw: u64) -> Self {
        Alpha {
            input_color_a: InputColor::from_u64(raw.bits(0..=1))
                .unwrap_or_else(|| panic!("Invalid input color A {:b}", raw.bits(0..=1))),
            input_color_b: InputColor::from_u64(raw.bits(2..=3))
                .unwrap_or_else(|| panic!("Invalid input color B {:b}", raw.bits(2..=3))),
            input_alpha_c: InputAlpha::from_u64(raw.bits(4..=5))
                .unwrap_or_else(|| panic!("Invalid input alpha C {:b}", raw.bits(4..=5))),
            input_color_d: InputColor::from_u64(raw.bits(6..=7))
                .unwrap_or_else(|| panic!("Invalid input color D {:b}", raw.bits(6..=7))),
            fixed: raw.bits(32..=39) as u8,
        }
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputColor {
    #[default]
    Source = 0b00, // Cs
    Destination = 0b01, // Cd
    Zero = 0b10,        // 0
    Reserved = 0b11,
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputAlpha {
    #[default]
    Source = 0b00, // As
    Destination = 0b01, // Ad
    Fixed = 0b10,       // FIX
    Reserved = 0b11,
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ZStorageFormat {
    #[default]
    Z32 = 0b0000,
    Z24 = 0b0001,
    Z16 = 0b0010,
    Z16s = 0b1010,
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PixelStorageFormat {
    #[default]
    Ct32 = 0b000000,
    Ct24 = 0b000001,
    Ct16 = 0b000010,
    Ct16s = 0b001010,
    Gpu24 = 0b010010,
    T8 = 0b010011,
    T4 = 0b010100,
    T8h = 0b011011,
    T4hl = 0b100100,
    T4hh = 0b101100,
    Z32 = 0b110000,
    Z24 = 0b110001,
    Z16 = 0b110010,
    Z16s = 0b111010,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct XyOffset {
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
pub struct Scissor {
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
pub struct Primitive {
    pub type_: PrimitiveType,                               // PRIM
    pub shading_method: ShadingMethod,                      // IIP
    pub texture_mapping: bool,                              // TME
    pub fogging: bool,                                      // FGE
    pub alpha_blending: bool,                               // ABE
    pub anti_aliasing: bool,                                // AA1
    pub texture_coordinate_method: TextureCoordinateMethod, // FST
    pub context: Context,                                   // CTXT
    pub fragment_value_control: FragmentValueControl,       // FIX
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
pub enum PrimitiveType {
    #[default]
    Point = 0b000,
    Line = 0b001,
    LineStrip = 0b010,
    Triangle = 0b011,
    TriangleStrip = 0b100,
    TriangleFan = 0b101,
    Sprite = 0b110,
    SpecificationProhibited = 0b111,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ShadingMethod {
    #[default]
    Flat,
    Gouraud,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TextureCoordinateMethod {
    #[default]
    Stq,
    Uv,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Context {
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
pub enum FragmentValueControl {
    #[default]
    Unfixed,
    Fixed,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rgbaq {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub q: f32,
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
pub struct Xyz {
    pub x: Fix124,
    pub y: Fix124,
    pub z: u32,
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
pub struct Uv {
    pub u: Fix124,
    pub v: Fix124,
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
pub struct BitBlitBuffer {
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
            source_base_pointer: value.bits(0..=13) as u32 * 64 * 4,
            source_width: value.bits(16..=21) as u16 * 64,
            source_pixel_storage_format: PixelStorageFormat::from_u64(value.bits(24..=29))
                .unwrap_or_else(|| {
                    panic!("Invalid pixel storage format {:b}", value.bits(24..=29))
                }),
            destination_base_pointer: value.bits(32..=45) as u32 * 64 * 4,
            destination_width: value.bits(48..=53) as u16 * 64,
            destination_pixel_storage_format: PixelStorageFormat::from_u64(value.bits(56..=61))
                .unwrap_or_else(|| {
                    panic!("Invalid pixel storage format {:b}", value.bits(56..=61))
                }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TransmissionPosition {
    source_x: u16,                 // SSAX
    source_y: u16,                 // SSAY
    destination_x: u16,            // DSAX
    destination_y: u16,            // DSAY
    order: PixelTransmissionOrder, // DIR
}

#[derive(Debug, Clone, Copy, Default, FromPrimitive)]
pub enum PixelTransmissionOrder {
    #[default]
    UpperLeftToLowerRight = 0b00,
    LowerLeftToUpperRight = 0b01,
    UpperRightToLowerLeft = 0b10,
    LowerRightToUpperLeft = 0b11,
}

impl From<u64> for TransmissionPosition {
    fn from(raw: u64) -> Self {
        TransmissionPosition {
            source_x: raw.bits(0..=10) as u16,
            source_y: raw.bits(16..=26) as u16,
            destination_x: raw.bits(32..=42) as u16,
            destination_y: raw.bits(48..=58) as u16,
            order: PixelTransmissionOrder::from_u64(raw.bits(59..=60)).unwrap_or_else(|| {
                panic!("Invalid pixel transmission order {:b}", raw.bits(59..=60))
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TransmissionSize {
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
    HostToLocal = 0b00,
    LocalToHost = 0b01,
    LocalToLocal = 0b10,
    Deactivated = 0b11,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DitherControl {
    enabled: bool,
}

impl From<u64> for DitherControl {
    fn from(raw: u64) -> Self {
        DitherControl {
            enabled: raw.bit(0),
        }
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, Default)]
pub enum ColorClamp {
    #[default]
    Mask,
    Clamp,
}

impl From<u64> for ColorClamp {
    fn from(raw: u64) -> Self {
        ColorClamp::from_u64(raw.bits(0..=0))
            .unwrap_or_else(|| panic!("Invalid color clamp {:b}", raw.bits(0..=0)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PrimitiveModeControl {
    #[default]
    PrimitiveMode, // Use PRMODE register
    Primitive, // Use PRIM register
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PixelTest {
    pub alpha_test: AlphaTest,        // ATE + ATST
    pub alpha_reference: u8,          // AREF
    pub alpha_fail: AlphaFail,        // AFAIL
    pub destination_alpha_test: bool, // DATE
    pub destination_alpha_mode: bool, // DATM
    pub depth_test: DepthTest,        // ZTE + ZTST
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
                false => DepthTest::Always, // Not allowed according the the spec, but I'll allow it.
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
    Never = 0b00,
    #[default]
    Always = 0b01,
    GreaterOrEqual = 0b10,
    Greater = 0b11,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Texture {
    pub base_pointer: u32,                               // TBP0
    pub buffer_width: u16,                               // TBW
    pub pixel_storage_format: PixelStorageFormat,        // PSM
    pub width: u16,                                      // TW
    pub height: u16,                                     // TH
    pub has_alpha: bool,                                 // TCC
    pub function: TextureFunction,                       // TFX
    pub clut_buffer_base_pointer: u32,                   // CBP
    pub clut_pixel_storage_format: PixelStorageFormat,   // CPSM
    pub clut_storage_mode: ClutStorageMode,              // CSM
    pub clut_entry_offset: u16,                          // CSA
    pub clut_buffer_load_control: ClutBufferLoadControl, // CLD
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
            base_pointer: raw.bits(0..=13) as u32 * 64 * 4,
            buffer_width: raw.bits(14..=19) as u16 * 64,
            pixel_storage_format: PixelStorageFormat::from_u64(raw.bits(20..=25))
                .unwrap_or_else(|| panic!("Invalid pixel storage format {:b}", raw.bits(20..=25))),
            width: 2u16.pow(raw.bits(26..=29) as _),
            height: 2u16.pow(raw.bits(30..=33) as _),
            has_alpha: raw.bit(34),
            function: TextureFunction::from_u64(raw.bits(35..=36))
                .unwrap_or_else(|| panic!("Invalid texture function {:b}", raw.bits(35..=36))),
            clut_buffer_base_pointer: raw.bits(37..=50) as u32 * 64 * 4,
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
