use std::{
    collections::VecDeque,
    ops::{Add, Sub},
};

use enum_map::Enum;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{bits::Bits, bytes::Bytes, fifo::Fifo};

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pub command_queue: VecDeque<(Register, u64)>,
    privileged_registers: PrivilegedRegisters,
    registers: Registers,
    vertex_queue: Fifo<Vertex>,
}

#[derive(Debug, Clone)]
struct Vertex {
    position: Xyz,
    color: Rgbaq,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Fix124 {
    raw: u16,
}

impl Fix124 {
    pub fn round(self) -> u16 {
        (self.raw + 8).bits(4..16)
    }

    pub fn floor(self) -> u16 {
        self.raw.bits(4..16)
    }

    pub fn ceil(self) -> u16 {
        (self.raw + 15).bits(4..16)
    }
}

impl Add for Fix124 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Fix124 {
            raw: self.raw + rhs.raw,
        }
    }
}

impl Sub for Fix124 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Fix124 {
            raw: self.raw - rhs.raw,
        }
    }
}

impl From<u16> for Fix124 {
    fn from(x: u16) -> Self {
        Fix124 { raw: x << 4 }
    }
}

impl From<Fix124> for f32 {
    fn from(x: Fix124) -> Self {
        x.raw as f32 / (1.0 * 16.0)
    }
}

impl From<f32> for Fix124 {
    fn from(x: f32) -> Self {
        Fix124 {
            raw: (x * 16.0) as u16,
        }
    }
}

#[derive(Debug, Default)]
struct PrivilegedRegisters {
    pcrtc_mode: u64,                           // PMODE
    sync_mode1: u64,                           // SMODE1
    sync_mode2: u64,                           // SMODE2
    dram_refresh: u64,                         // SRFSH
    synch1: u64,                               // SYNCH1
    synch2: u64,                               // SYNCH2
    syncv: u64,                                // SYNCV
    display_frame_buffer1: DisplayFrameBuffer, // DISPFB1
    display1: Display,                         // DISPLAY1
    display_frame_buffer2: DisplayFrameBuffer, // DISPFB1
    display2: Display,                         // DISPLAY1
    write_buffer: u64,                         // EXTBUF
    write_data: u64,                           // EXTDATA
    write_start: u64,                          // EXTWRITE
    background_color: u64,                     // BGCOLOR
    status: u64,                               // CSR
    interrupt_mask: u64,                       // IMR
    bus_direction: u64,                        // BUSDIR
    signal_label_id: u64,                      // SIGLBLID
}

#[derive(Debug, Default)]
struct Registers {
    primitive: Primitive,                          // PRIM
    rgbaq: Rgbaq,                                  // RGBAQ
    xyz: Xyz,                                      // XYZ2
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

    pub fn frame_buffer(&self) -> Option<(u16, Vec<u8>)> {
        for frame_buffer in [
            &self.privileged_registers.display_frame_buffer1,
            &self.privileged_registers.display_frame_buffer2,
        ] {
            if frame_buffer.width == 0 {
                continue;
            }
            assert!(frame_buffer.offset_x == 0 && frame_buffer.offset_y == 0);
            assert!(frame_buffer.pixel_storage_format == PixelStorageFormat::Psmct32);
            let mut result = Vec::with_capacity(frame_buffer.width as usize * 4 * 480);
            for y in 0..480 {
                for x in 0..frame_buffer.width {
                    result.extend_from_slice(
                        &self
                            .read_psmct32(frame_buffer.base_pointer, x, y, frame_buffer.width)
                            .to_bytes(),
                    );
                }
            }
            return Some((frame_buffer.width, result));
        }
        None
    }

    fn psmct32_offset(x: u16, y: u16, width: u16) -> u32 {
        let page_x = x / 64;
        let page_y = y / 32;
        let page_index = page_y as u32 * (width as u32 / 64) + page_x as u32;
        let page_size = 8192;
        let page_offset = page_index * page_size;
        let block_x = (x % 64) / 8;
        let block_y = (y % 32) / 8;
        let block_index = block_x.bit(0).then_some(1).unwrap_or_default()
            | block_x.bit(1).then_some(4).unwrap_or_default()
            | block_x.bit(2).then_some(16).unwrap_or_default()
            | block_y.bit(0).then_some(2).unwrap_or_default()
            | block_y.bit(1).then_some(8).unwrap_or_default();
        let block_size = 256;
        let block_offset = block_index * block_size;
        let local_x = x % 8;
        let local_y = y % 8;
        let local_index = local_x.bit(0).then_some(1).unwrap_or_default()
            | local_x.bit(1).then_some(4).unwrap_or_default()
            | local_x.bit(2).then_some(8).unwrap_or_default()
            | local_y.bit(0).then_some(2).unwrap_or_default()
            | local_y.bit(1).then_some(16).unwrap_or_default()
            | local_y.bit(2).then_some(32).unwrap_or_default();
        let pixel_size = 4;
        let local_offset = local_index * pixel_size;
        page_offset + block_offset + local_offset
    }

    fn read_psmct32(&self, base_pointer: u32, x: u16, y: u16, width: u16) -> u32 {
        let address = base_pointer + Self::psmct32_offset(x, y, width);
        u32::from_bytes(&self.local_memory[address as usize..address as usize + 4])
    }

    fn write_psmct32(&mut self, base_pointer: u32, x: u16, y: u16, width: u16, value: u32) {
        let address = base_pointer + Self::psmct32_offset(x, y, width);
        self.local_memory[address as usize..address as usize + 4]
            .copy_from_slice(&value.to_bytes());
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
                Register::Uv => todo!(),
                Register::Xyzf2 => todo!(),
                Register::Xyz2 => {
                    self.registers.xyz = Xyz::from(data);
                    self.vertex_kick(/* drawing_kick */ true);
                }
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
                Register::TextureFlush => todo!(),
                Register::Scissor1 => self.registers.contextual[0].scissor = Scissor::from(data),
                Register::Scissor2 => self.registers.contextual[1].scissor = Scissor::from(data),
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

    pub fn vertex_kick(&mut self, drawing_kick: bool) {
        let vertex = Vertex {
            position: Xyz {
                x: self.registers.xyz.x - self.contextual_registers().xy_offset.x,
                y: self.registers.xyz.y - self.contextual_registers().xy_offset.y,
                z: self.registers.xyz.z,
            },
            color: self.registers.rgbaq,
        };

        match self.registers.primitive.type_ {
            PrimitiveType::Point => {
                self.vertex_queue.clear();
                if drawing_kick {
                    self.draw_point(&vertex);
                }
            }
            PrimitiveType::Line => {
                if let Some(vertex1) = self.vertex_queue.pop_back() {
                    self.vertex_queue.clear();
                    if drawing_kick {
                        self.draw_line(&vertex1, &vertex);
                    }
                } else {
                    self.vertex_queue.push_back(vertex);
                }
            }
            PrimitiveType::LineStrip => {
                if let Some(vertex1) = self.vertex_queue.pop_back() {
                    if drawing_kick {
                        self.draw_line(&vertex1, &vertex);
                    }
                    self.vertex_queue.clear();
                }
                self.vertex_queue.push_back(vertex);
            }
            PrimitiveType::Triangle => {
                if self.vertex_queue.len() == 2 {
                    let vertex2 = self.vertex_queue.pop_back().unwrap();
                    let vertex1 = self.vertex_queue.pop_back().unwrap();
                    if drawing_kick {
                        self.draw_triangle(&vertex1, &vertex2, &vertex);
                    }
                } else {
                    self.vertex_queue.push_back(vertex);
                }
            }
            PrimitiveType::TriangleStrip => {
                if self.vertex_queue.len() == 2 {
                    let vertex1 = self.vertex_queue.pop_front().unwrap();
                    let vertex2 = self.vertex_queue.front().unwrap().clone();
                    if drawing_kick {
                        self.draw_triangle(&vertex1, &vertex2, &vertex);
                    }
                }
                self.vertex_queue.push_back(vertex);
            }
            PrimitiveType::TriangleFan => {
                if self.vertex_queue.len() == 2 {
                    let vertex1 = self.vertex_queue.front().unwrap().clone();
                    let vertex2 = self.vertex_queue.pop_back().unwrap();
                    if drawing_kick {
                        self.draw_triangle(&vertex1, &vertex2, &vertex);
                    }
                }
                self.vertex_queue.push_back(vertex);
            }
            PrimitiveType::Sprite => {
                if let Some(vertex1) = self.vertex_queue.pop_back() {
                    self.vertex_queue.clear();
                    if drawing_kick {
                        self.draw_sprite(&vertex1, &vertex);
                    }
                } else {
                    self.vertex_queue.push_back(vertex);
                }
            }
            PrimitiveType::SpecificationProhibited => todo!(),
        }
    }

    fn draw_point(&mut self, vertex: &Vertex) {
        let scissor = self.contextual_registers().scissor;
        let pixel_x = vertex.position.x.round();
        let pixel_y = vertex.position.y.round();
        // println!("Draw point: {vertex:?}=({pixel_x}, {pixel_y})");
        if !scissor.contains(pixel_x, pixel_y) {
            // println!("Point outside scissor: {:?}", scissor);
            return;
        }
        let frame = self.contextual_registers().frame_buffer_settings;
        match frame.pixel_storage_format {
            PixelStorageFormat::Psmct32 => {
                self.write_psmct32(
                    frame.base_pointer,
                    pixel_x,
                    pixel_y,
                    frame.width,
                    u32::from_bytes(&[
                        vertex.color.r,
                        vertex.color.g,
                        vertex.color.b,
                        vertex.color.a,
                    ]),
                );
            }
            _ => todo!(),
        }
        // TODO scan mask
        // TODO texturing
        // TODO depth test
        // TODO alpha
        // TODO z update
        // TODO drawing mask
    }

    fn clip_line(&self, start: Xyz, end: Xyz) -> Option<(f32, f32)> {
        let scissor = self.contextual_registers().scissor;
        let sx0 = f32::from(scissor.x0);
        let sy0 = f32::from(scissor.y0);
        let sx1 = f32::from(scissor.x1 + 1);
        let sy1 = f32::from(scissor.y1 + 1);
        let vx0 = f32::from(start.x);
        let vy0 = f32::from(start.y);
        let vx1 = f32::from(end.x);
        let vy1 = f32::from(end.y);
        let dx = vx1 - vx0;
        let dy = vy1 - vy0;

        let mut t0: f32 = 0.0;
        let mut t1: f32 = 1.0;

        if dx == 0.0 {
            if !(sx0..sx1).contains(&vx0) {
                return None;
            }
        } else {
            let mut p = (sx0 - vx0) / dx;
            let mut q = (sx1 - vx0) / dx;
            if dx < 0.0 {
                std::mem::swap(&mut p, &mut q);
            }
            t0 = t0.max(p);
            t1 = t1.min(q);
            if t0 > t1 {
                return None;
            }
        }

        if dy == 0.0 {
            if !(sy0..sy1).contains(&vy0) {
                return None;
            }
        } else {
            let mut p = (sy0 - vy0) / dy;
            let mut q = (sy1 - vy0) / dy;
            if dy < 0.0 {
                std::mem::swap(&mut p, &mut q);
            }
            t0 = t0.max(p);
            t1 = t1.min(q);
            if t0 > t1 {
                return None;
            }
        }

        Some((t0, t1))
    }

    fn draw_line(&mut self, start: &Vertex, end: &Vertex) {
        // println!("Draw line: {:?} {:?}", start, end);
        let Some((t0, t1)) = self.clip_line(start.position, end.position) else {
            return;
        };
        let dx = f32::from(end.position.x) - f32::from(start.position.x);
        let dy = f32::from(end.position.y) - f32::from(start.position.y);
        let dr = end.color.r as i32 - start.color.r as i32;
        let dg = end.color.g as i32 - start.color.g as i32;
        let db = end.color.b as i32 - start.color.b as i32;
        let da = end.color.a as i32 - start.color.a as i32;
        let pixels = dx.abs().max(dy.abs()).round();
        let start_pixel = (t0 * pixels) as i32;
        let end_pixel = (t1 * pixels) as i32;

        let frame = self.contextual_registers().frame_buffer_settings;
        match frame.pixel_storage_format {
            PixelStorageFormat::Psmct32 => {
                for i in start_pixel..end_pixel {
                    let pixel_x =
                        (f32::from(start.position.x) + dx * i as f32 / pixels).round() as u16;
                    let pixel_y =
                        (f32::from(start.position.y) + dy * i as f32 / pixels).round() as u16;
                    let r = (start.color.r as i32 + dr * i / pixels as i32) as u8;
                    let g = (start.color.g as i32 + dg * i / pixels as i32) as u8;
                    let b = (start.color.b as i32 + db * i / pixels as i32) as u8;
                    let a = (start.color.a as i32 + da * i / pixels as i32) as u8;
                    self.write_psmct32(
                        frame.base_pointer,
                        pixel_x,
                        pixel_y,
                        frame.width,
                        u32::from_bytes(&[r, g, b, a]),
                    );
                }
            }
            _ => todo!(),
        }
    }

    fn draw_triangle(&mut self, vertex1: &Vertex, vertex2: &Vertex, vertex3: &Vertex) {
        todo!()
    }

    fn draw_sprite(&mut self, vertex1: &Vertex, vertex2: &Vertex) {
        let scissor = self.contextual_registers().scissor;
        let x0 = vertex1.position.x.ceil().clamp(scissor.x0, scissor.x1) as i32;
        let mut x1 = vertex2.position.x.ceil().clamp(scissor.x0, scissor.x1 + 1) as i32;
        let y0 = vertex1.position.y.ceil().clamp(scissor.y0, scissor.y1) as i32;
        let mut y1 = vertex2.position.y.ceil().clamp(scissor.y0, scissor.y1 + 1) as i32;
        let frame = self.contextual_registers().frame_buffer_settings;
        if x1 == x0 || y1 == y0 {
            return;
        }

        x1 -= 1;
        y1 -= 1;
        // TODO: interpolate color
        match frame.pixel_storage_format {
            PixelStorageFormat::Psmct32 => {
                for y in y0.min(y1)..=y1.max(y0) {
                    for x in x0.min(x1)..=x1.max(x0) {
                        self.write_psmct32(
                            frame.base_pointer,
                            x as u16,
                            y as u16,
                            frame.width,
                            u32::from_bytes(&[
                                vertex1.color.r,
                                vertex1.color.g,
                                vertex1.color.b,
                                vertex1.color.a,
                            ]),
                        );
                    }
                }
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Default)]
struct Display {
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

#[derive(Debug, Default)]
struct DisplayFrameBuffer {
    base_pointer: u32,
    width: u16,
    pixel_storage_format: PixelStorageFormat,
    offset_x: u16,
    offset_y: u16,
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

#[repr(u8)]
#[derive(FromPrimitive, Debug, Enum)]
pub enum Register {
    Primitive = 0x00,              // PRIM Drawing primitive setting
    Rgbaq = 0x01,                  // RGBAQ Vertex color setting
    St = 0x02,                     // ST Vertex texture coordinate setting (texture coordinates)
    Uv = 0x03,                     // UV Vertex texture coordinate setting (texel coordinates)
    Xyzf2 = 0x04,                  // XYZF2 Vertex coordinate value setting
    Xyz2 = 0x05,                   // XYZ2 Vertex coordinate value setting
    Tex0_1 = 0x06,                 // TEX0_1 Texture information setting
    Tex0_2 = 0x07,                 // TEX0_2 Texture information setting
    Clamp1 = 0x08,                 // CLAMP_1 Texture wrap mode
    Clamp2 = 0x09,                 // CLAMP_2 Texture wrap mode
    Fog = 0x0a,                    // FOG Vertex fog value setting
    Xyzf3 = 0x0c,                  // XYZF3 Vertex coordinate value setting (without drawing kick)
    Xyz3 = 0x0d,                   // XYZ3 Vertex coordinate value setting (without drawing kick)
    Texture1_1 = 0x14,             // TEX1_1 Texture information setting
    Texture1_2 = 0x15,             // TEX1_2 Texture information setting
    Texture2_1 = 0x16,             // TEX2_1 Texture information setting
    Texture2_2 = 0x17,             // TEX2_2 Texture information setting
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
            pixel_storage_format: PixelStorageFormat::from_u8(raw.bits(24..=29) as u8)
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
            x: Fix124 {
                raw: raw.bits(0..16) as u16,
            },
            y: Fix124 {
                raw: raw.bits(32..48) as u16,
            },
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
            x: Fix124 {
                raw: raw.bits(0..16) as u16,
            },
            y: Fix124 {
                raw: raw.bits(16..32) as u16,
            },
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

#[derive(Debug, Clone, Copy, Default)]
pub enum PrimitiveModeControl {
    #[default]
    PrimitiveMode, // Use PRMODE register
    Primitive, // Use PRIM register
}
