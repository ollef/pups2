use std::ops::{Add, AddAssign, Div, Mul, Sub};

use num_traits::AsPrimitive;

use crate::{bytes::Bytes, emotion_engine::gs::registers::PixelStorageFormat, fix::Fix};

use super::{
    registers::{PrimitiveType, Rgbaq, Uv, Xyz},
    Gs,
};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Xyz,
    pub color: Rgbaq,
    pub uv: Uv,
}

#[derive(Debug, Clone)]
pub struct FloatVertex {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    q: f32,
    u: f32,
    v: f32,
}

impl FloatVertex {
    pub fn xy(&self) -> Xy {
        Xy {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<&Vertex> for FloatVertex {
    fn from(v: &Vertex) -> Self {
        FloatVertex {
            x: f32::from(v.position.x),
            y: f32::from(v.position.y),
            z: v.position.z as f32,
            r: v.color.r as f32,
            g: v.color.g as f32,
            b: v.color.b as f32,
            a: v.color.a as f32,
            q: v.color.q,
            u: f32::from(v.uv.u),
            v: f32::from(v.uv.v),
        }
    }
}

impl Sub for &FloatVertex {
    type Output = FloatVertex;

    fn sub(self, rhs: Self) -> Self::Output {
        FloatVertex {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
            q: self.q - rhs.q,
            u: self.u - rhs.u,
            v: self.v - rhs.v,
        }
    }
}

impl Sub for FloatVertex {
    type Output = FloatVertex;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl Add for &FloatVertex {
    type Output = FloatVertex;

    fn add(self, rhs: Self) -> Self::Output {
        FloatVertex {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
            q: self.q + rhs.q,
            u: self.u + rhs.u,
            v: self.v + rhs.v,
        }
    }
}

impl Add for FloatVertex {
    type Output = FloatVertex;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl AddAssign<&FloatVertex> for FloatVertex {
    fn add_assign(&mut self, rhs: &Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
        self.q += rhs.q;
        self.u += rhs.u;
        self.v += rhs.v;
    }
}

impl Mul<f32> for &FloatVertex {
    type Output = FloatVertex;

    fn mul(self, rhs: f32) -> Self::Output {
        FloatVertex {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
            q: self.q * rhs,
            u: self.u * rhs,
            v: self.v * rhs,
        }
    }
}

impl Mul<f32> for FloatVertex {
    type Output = FloatVertex;

    fn mul(self, rhs: f32) -> Self::Output {
        &self * rhs
    }
}

impl Div<f32> for &FloatVertex {
    type Output = FloatVertex;

    fn div(self, rhs: f32) -> Self::Output {
        FloatVertex {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
            a: self.a / rhs,
            q: self.q / rhs,
            u: self.u / rhs,
            v: self.v / rhs,
        }
    }
}

impl Div<f32> for FloatVertex {
    type Output = FloatVertex;

    fn div(self, rhs: f32) -> Self::Output {
        &self / rhs
    }
}

#[derive(Debug, Clone, Copy)]
struct Xy {
    x: f32,
    y: f32,
}

impl Mul<f32> for Xy {
    type Output = Xy;

    fn mul(self, rhs: f32) -> Self::Output {
        Xy {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
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

    pub fn vertex_kick(&mut self, drawing_kick: bool) {
        let vertex = Vertex {
            position: Xyz {
                x: self.registers.xyz.x - self.contextual_registers().xy_offset.x,
                y: self.registers.xyz.y - self.contextual_registers().xy_offset.y,
                z: self.registers.xyz.z,
            },
            color: self.registers.rgbaq,
            uv: self.registers.uv,
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

    fn clip_line(&self, start: &FloatVertex, delta: &FloatVertex) -> Option<(f32, f32)> {
        let scissor = self.contextual_registers().scissor;
        let mut t0: f32 = 0.0;
        let mut t1: f32 = 1.0;

        if delta.x == 0.0 {
            if !(f32::from(scissor.x0)..f32::from(scissor.x1) + 1.0).contains(&start.x) {
                return None;
            }
        } else {
            let mut p = (f32::from(scissor.x0) - start.x) / delta.x;
            let mut q = (f32::from(scissor.x1) + 1.0 - start.x) / delta.x;
            if delta.x < 0.0 {
                std::mem::swap(&mut p, &mut q);
            }
            t0 = t0.max(p);
            t1 = t1.min(q);
            if t0 > t1 {
                return None;
            }
        }

        if delta.y == 0.0 {
            if !(f32::from(scissor.y0)..f32::from(scissor.y1) + 1.0).contains(&start.y) {
                return None;
            }
        } else {
            let mut p = (f32::from(scissor.y0) - start.y) / delta.y;
            let mut q = (f32::from(scissor.y1) + 1.0 - start.y) / delta.y;
            if delta.y < 0.0 {
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
        let start = FloatVertex::from(start);
        let end = FloatVertex::from(end);
        let delta = &end - &start;

        // println!("Draw line: {:?} {:?}", start, end);
        let Some((t0, t1)) = self.clip_line(&start, &delta) else {
            return;
        };
        let pixels = delta.x.abs().max(delta.y.abs()).round();
        let delta_pixel = &delta / pixels;
        let start_pixel = (t0 * pixels) as i32;
        let end_pixel = (t1 * pixels) as i32;

        let frame = self.contextual_registers().frame_buffer_settings;
        match frame.pixel_storage_format {
            PixelStorageFormat::Psmct32 => {
                let mut v = &start + &(&delta_pixel * start_pixel as f32);
                for _ in start_pixel..end_pixel {
                    self.write_psmct32(
                        frame.base_pointer,
                        v.x as u16,
                        v.y as u16,
                        frame.width,
                        u32::from_bytes(&[v.r as u8, v.g as u8, v.b as u8, v.a as u8]),
                    );
                    v += &delta_pixel;
                }
            }
            _ => todo!(),
        }
    }

    fn edge(v1: Xy, v2: Xy, v3: Xy) -> f32 {
        (v2.x - v1.x) * (v3.y - v1.y) - (v2.y - v1.y) * (v3.x - v1.x)
    }

    fn render_triangle(&mut self, vertex1: &Vertex, vertex2: &Vertex, vertex3: &Vertex) {
        let vertex1 = FloatVertex::from(vertex1);
        let vertex2 = FloatVertex::from(vertex2);
        let vertex3 = FloatVertex::from(vertex3);
        let area = Self::edge(vertex1.xy(), vertex2.xy(), vertex3.xy()); // Signed (twice the) triangle area
        if area == 0.0 {
            return;
        }
        // Ensure counter-clockwise winding order.
        let (area, vertex2, vertex3) = if area < 0.0 {
            (-area, vertex3, vertex2)
        } else {
            (area, vertex2, vertex3)
        };

        let scissor = self.contextual_registers().scissor;
        let min_pixel = Xy {
            x: vertex1
                .x
                .min(vertex2.x)
                .min(vertex3.x)
                .ceil()
                .max(scissor.x0 as f32),
            y: vertex1
                .y
                .min(vertex2.y)
                .min(vertex3.y)
                .ceil()
                .max(scissor.y0 as f32),
        };
        let max_pixel = Xy {
            x: vertex1
                .x
                .max(vertex2.x)
                .max(vertex3.x)
                .ceil()
                .min(scissor.x1 as f32 + 1.0),
            y: vertex1
                .y
                .max(vertex2.y)
                .max(vertex3.y)
                .ceil()
                .min(scissor.y1 as f32 + 1.0),
        };

        if min_pixel.x >= max_pixel.x || min_pixel.y >= max_pixel.y {
            return;
        }

        let inv_area = 1.0 / area;

        let mut w1_start = Self::edge(vertex2.xy(), vertex3.xy(), min_pixel) * inv_area;
        let mut w2_start = Self::edge(vertex3.xy(), vertex1.xy(), min_pixel) * inv_area;
        let mut w3_start = Self::edge(vertex1.xy(), vertex2.xy(), min_pixel) * inv_area;
        let delta = |p1: Xy, p2: Xy| Xy {
            x: p1.y - p2.y,
            y: p2.x - p1.x,
        };
        let w1_delta = delta(vertex2.xy(), vertex3.xy()) * inv_area;
        let w2_delta = delta(vertex3.xy(), vertex1.xy()) * inv_area;
        let w3_delta = delta(vertex1.xy(), vertex2.xy()) * inv_area;
        let bias = |p1: Xy, p2: Xy| {
            if (p1.y, p2.x) < (p2.y, p1.x) {
                1.0 / 16.0
            } else {
                0.0
            }
        };
        let bias1 = bias(vertex2.xy(), vertex3.xy()) * inv_area;
        let bias2 = bias(vertex3.xy(), vertex1.xy()) * inv_area;
        let bias3 = bias(vertex1.xy(), vertex2.xy()) * inv_area;

        let frame = self.contextual_registers().frame_buffer_settings;

        for y in min_pixel.y as u16..max_pixel.y as u16 {
            let mut w1 = w1_start;
            let mut w2 = w2_start;
            let mut w3 = w3_start;

            for x in min_pixel.x as u16..max_pixel.x as u16 {
                if w1 >= bias1 && w2 >= bias2 && w3 >= bias3 {
                    let vertex = &vertex1 * w1 + &vertex2 * w2 + &vertex3 * w3;

                    match frame.pixel_storage_format {
                        PixelStorageFormat::Psmct32 => {
                            self.write_psmct32(
                                frame.base_pointer,
                                x,
                                y,
                                frame.width,
                                u32::from_bytes(&[
                                    vertex.r as u8,
                                    vertex.g as u8,
                                    vertex.b as u8,
                                    vertex.a as u8,
                                ]),
                            );
                        }
                        _ => todo!(),
                    }
                }

                w1 += w1_delta.x;
                w2 += w2_delta.x;
                w3 += w3_delta.x;
            }

            w1_start += w1_delta.y;
            w2_start += w2_delta.y;
            w3_start += w3_delta.y;
        }
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
