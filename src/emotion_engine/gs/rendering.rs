use crate::{bits::Bits, bytes::Bytes};

use super::{Gs, PixelStorageFormat, PrimitiveType, Rgbaq, Xyz};

#[derive(Debug, Clone)]
pub struct Vertex {
    position: Xyz,
    color: Rgbaq,
}

impl Gs {
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

    pub fn write_psmct32(&mut self, base_pointer: u32, x: u16, y: u16, width: u16, value: u32) {
        let address = base_pointer + Self::psmct32_offset(x, y, width);
        self.local_memory[address as usize..address as usize + 4]
            .copy_from_slice(&value.to_bytes());
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
                    self.render_point(&vertex);
                }
            }
            PrimitiveType::Line => {
                if let Some(vertex1) = self.vertex_queue.pop_back() {
                    self.vertex_queue.clear();
                    if drawing_kick {
                        self.render_line(&vertex1, &vertex);
                    }
                } else {
                    self.vertex_queue.push_back(vertex);
                }
            }
            PrimitiveType::LineStrip => {
                if let Some(vertex1) = self.vertex_queue.pop_back() {
                    if drawing_kick {
                        self.render_line(&vertex1, &vertex);
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
                        self.render_triangle(&vertex1, &vertex2, &vertex);
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
                        self.render_triangle(&vertex1, &vertex2, &vertex);
                    }
                }
                self.vertex_queue.push_back(vertex);
            }
            PrimitiveType::TriangleFan => {
                if self.vertex_queue.len() == 2 {
                    let vertex1 = self.vertex_queue.front().unwrap().clone();
                    let vertex2 = self.vertex_queue.pop_back().unwrap();
                    if drawing_kick {
                        self.render_triangle(&vertex1, &vertex2, &vertex);
                    }
                }
                self.vertex_queue.push_back(vertex);
            }
            PrimitiveType::Sprite => {
                if let Some(vertex1) = self.vertex_queue.pop_back() {
                    self.vertex_queue.clear();
                    if drawing_kick {
                        self.render_sprite(&vertex1, &vertex);
                    }
                } else {
                    self.vertex_queue.push_back(vertex);
                }
            }
            PrimitiveType::SpecificationProhibited => todo!(),
        }
    }

    fn render_point(&mut self, vertex: &Vertex) {
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

    fn render_line(&mut self, start: &Vertex, end: &Vertex) {
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

    fn render_triangle(&mut self, vertex1: &Vertex, vertex2: &Vertex, vertex3: &Vertex) {
        todo!()
    }

    fn render_sprite(&mut self, vertex1: &Vertex, vertex2: &Vertex) {
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
