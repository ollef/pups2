use std::ops::{Add, AddAssign, Div, Mul, Sub};

use num_traits::AsPrimitive;

use crate::{
    bits::Bits,
    bytes::Bytes,
    emotion_engine::gs::{
        privileged_registers::{AlphaBlendingMethod, AlphaValueSelection, Rgb},
        registers::PixelStorageFormat,
    },
    fix::Fix,
};

use super::{
    registers::{
        ColorClamp, DepthTest, InputAlpha, InputColor, PrimitiveType, Rgbaq,
        TextureCoordinateMethod, TextureFunction, Uv, Xyz, ZStorageFormat,
    },
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<u32> for Rgba {
    fn from(raw: u32) -> Self {
        Rgba {
            r: raw.bits(0..8) as u8,
            g: raw.bits(8..16) as u8,
            b: raw.bits(16..24) as u8,
            a: raw.bits(24..32) as u8,
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
pub struct Xy {
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

#[derive(Default, Debug, Clone)]
pub struct Rect<T> {
    pub x_start: T,
    pub y_start: T,
    pub x_end: T,
    pub y_end: T,
}

impl<T: Copy> Rect<T> {
    pub fn is_empty(&self) -> bool
    where
        T: PartialOrd,
    {
        self.x_start >= self.x_end || self.y_start >= self.y_end
    }

    pub fn contains(&self, x: T, y: T) -> bool
    where
        T: PartialOrd,
    {
        self.x_start <= x && x < self.x_end && self.y_start <= y && y < self.y_end
    }

    pub fn width(&self) -> T
    where
        T: Sub<Output = T>,
    {
        self.x_end - self.x_start
    }

    pub fn height(&self) -> T
    where
        T: Sub<Output = T>,
    {
        self.y_end - self.y_start
    }

    pub fn area(&self) -> T
    where
        T: Sub<Output = T> + Mul<Output = T>,
    {
        self.width() * self.height()
    }

    pub fn union(&self, other: &Self) -> Self
    where
        T: Ord,
    {
        Rect {
            x_start: self.x_start.min(other.x_start),
            y_start: self.y_start.min(other.y_start),
            x_end: self.x_end.max(other.x_end),
            y_end: self.y_end.max(other.y_end),
        }
    }

    pub fn as_<T2>(&self) -> Rect<T2>
    where
        T: AsPrimitive<T2>,
        T2: Copy + 'static,
    {
        Rect {
            x_start: self.x_start.as_(),
            y_start: self.y_start.as_(),
            x_end: self.x_end.as_(),
            y_end: self.y_end.as_(),
        }
    }
}

impl Gs {
    pub fn frame_buffer(&self) -> Option<(u16, Vec<u8>)> {
        let pcrtc = self.privileged_registers.pcrtc_mode;
        let display1 = self.privileged_registers.display1;
        let display2 = self.privileged_registers.display2;
        let rect1 = pcrtc.enable_circuit1.then_some(display1.rect());
        let rect2 = pcrtc.enable_circuit2.then_some(display2.rect());
        let rect = match (&rect1, &rect2) {
            (Some(r1), Some(r2)) => r1.union(r2),
            (Some(r), None) | (None, Some(r)) => r.clone(),
            (None, None) => return None,
        };
        let rect1 = rect1.unwrap_or_default();
        let rect2 = rect2.unwrap_or_default();

        let fb1 = self.privileged_registers.display_frame_buffer1;
        let fb2 = self.privileged_registers.display_frame_buffer2;
        assert!(!pcrtc.enable_circuit1 || fb1.pixel_storage_format == PixelStorageFormat::Ct32);
        assert!(!pcrtc.enable_circuit2 || fb2.pixel_storage_format == PixelStorageFormat::Ct32);
        if rect.is_empty() {
            return None;
        }

        let mut result = Vec::with_capacity(rect.as_::<usize>().area() * 4);
        for y in rect.y_start..rect.y_end {
            for x in rect.x_start..rect.x_end {
                let in_rect1 = rect1.contains(x, y);
                let out1 = if pcrtc.enable_circuit1 && in_rect1 {
                    Rgba::from(self.read_psmct32(
                        fb1.base_pointer,
                        x + fb1.offset_x - rect1.x_start,
                        y + fb1.offset_y - rect1.y_start,
                        fb1.width,
                    ))
                } else {
                    Rgba::default()
                };
                let out2 = match pcrtc.alpha_blending_method {
                    AlphaBlendingMethod::Circuit2 => {
                        if pcrtc.enable_circuit2 && rect2.contains(x, y) {
                            Rgb::from(self.read_psmct32(
                                fb2.base_pointer,
                                x + fb2.offset_x - rect2.x_start,
                                y + fb2.offset_y - rect2.y_start,
                                fb2.width,
                            ))
                        } else {
                            self.privileged_registers.background_color
                        }
                    }
                    AlphaBlendingMethod::BackgroundColor => {
                        self.privileged_registers.background_color
                    }
                };

                let a = match pcrtc.alpha_value_selection {
                    AlphaValueSelection::Circuit1 => (out1.a as u32 * 2).min(255),
                    AlphaValueSelection::Fixed(a) => {
                        if in_rect1 {
                            a as u32
                        } else {
                            0
                        }
                    }
                };

                let r = ((out1.r as u32 * a + out2.r as u32 * (255 - a)) >> 8).min(255) as u8;
                let g = ((out1.g as u32 * a + out2.g as u32 * (255 - a)) >> 8).min(255) as u8;
                let b = ((out1.b as u32 * a + out2.b as u32 * (255 - a)) >> 8).min(255) as u8;
                result.extend_from_slice(&[b, g, r, 0]);
            }
        }

        Some((rect.width(), result))
    }

    pub fn vertex_kick(&mut self, drawing_kick: bool) {
        let vertex = Vertex {
            position: Xyz {
                x: self.registers.xyz.x,
                y: self.registers.xyz.y,
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

    fn relative(&self, v: &Vertex) -> FloatVertex {
        let xy_offset = self.contextual_registers().xy_offset;
        FloatVertex {
            x: f32::from(v.position.x) - f32::from(xy_offset.x),
            y: f32::from(v.position.y) - f32::from(xy_offset.y),
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

    fn render_point(&mut self, vertex: &Vertex) {
        let scissor = self.contextual_registers().scissor;
        let vertex = self.relative(vertex);
        let x = vertex.x as u16;
        let y = vertex.y as u16;
        // println!("Draw point: {vertex:?}=({pixel_x}, {pixel_y})");
        if !scissor.contains(x, y) {
            // println!("Point outside scissor: {:?}", scissor);
            return;
        }
        self.render_pixel(
            x,
            y,
            vertex.y as u32,
            Rgba {
                r: vertex.r as u8,
                g: vertex.g as u8,
                b: vertex.b as u8,
                a: vertex.a as u8,
            },
            Uv {
                u: Fix::from(vertex.u),
                v: Fix::from(vertex.v),
            },
        );
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
        let start = self.relative(start);
        let end = self.relative(end);
        let delta = &end - &start;

        // println!("Draw line: {:?} {:?}", start, end);
        let Some((t0, t1)) = self.clip_line(&start, &delta) else {
            return;
        };
        let pixels = delta.x.abs().max(delta.y.abs()).round();
        let delta_pixel = &delta / pixels;
        let start_pixel = (t0 * pixels) as i32;
        let end_pixel = (t1 * pixels) as i32;
        let mut v = &start + &(&delta_pixel * start_pixel as f32);
        for _ in start_pixel..end_pixel {
            self.render_pixel(
                v.x as u16,
                v.y as u16,
                v.z as u32,
                Rgba {
                    r: v.r as u8,
                    g: v.g as u8,
                    b: v.b as u8,
                    a: v.a as u8,
                },
                Uv {
                    u: Fix::from(v.u),
                    v: Fix::from(v.v),
                },
            );
            v += &delta_pixel;
        }
    }

    fn edge(v1: Xy, v2: Xy, v3: Xy) -> f32 {
        (v2.x - v1.x) * (v3.y - v1.y) - (v2.y - v1.y) * (v3.x - v1.x)
    }

    fn render_triangle(&mut self, vertex1: &Vertex, vertex2: &Vertex, vertex3: &Vertex) {
        let vertex1 = self.relative(vertex1);
        let vertex2 = self.relative(vertex2);
        let vertex3 = self.relative(vertex3);
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
            if (p2.y, p1.x) < (p1.y, p2.x) {
                1.0 / 16.0
            } else {
                0.0
            }
        };
        let bias1 = bias(vertex2.xy(), vertex3.xy()) * inv_area;
        let bias2 = bias(vertex3.xy(), vertex1.xy()) * inv_area;
        let bias3 = bias(vertex1.xy(), vertex2.xy()) * inv_area;

        for y in min_pixel.y as u16..max_pixel.y as u16 {
            let mut w1 = w1_start;
            let mut w2 = w2_start;
            let mut w3 = w3_start;

            for x in min_pixel.x as u16..max_pixel.x as u16 {
                if w1 >= bias1 && w2 >= bias2 && w3 >= bias3 {
                    let vertex = &vertex1 * w1 + &vertex2 * w2 + &vertex3 * w3;

                    self.render_pixel(
                        x,
                        y,
                        vertex.z as u32,
                        Rgba {
                            r: vertex.r as u8,
                            g: vertex.g as u8,
                            b: vertex.b as u8,
                            a: vertex.a as u8,
                        },
                        Uv {
                            u: Fix::from(vertex.u),
                            v: Fix::from(vertex.v),
                        },
                    );
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
        let color = vertex2.color;
        let vertex1 = self.relative(vertex1);
        let vertex2 = self.relative(vertex2);
        let scissor = self.contextual_registers().scissor;
        let x0 = vertex1.x.min(vertex2.x).ceil().max(f32::from(scissor.x0));
        let x1 = vertex1
            .x
            .max(vertex2.x)
            .ceil()
            .min(f32::from(scissor.x1 + 1));
        let y0 = vertex1.y.min(vertex2.y).ceil().max(f32::from(scissor.y0));
        let y1 = vertex1
            .y
            .max(vertex2.y)
            .ceil()
            .min(f32::from(scissor.y1 + 1));
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let w = vertex2.x - vertex1.x;
        let inv_w = 1.0 / w;
        let h = vertex2.y - vertex1.y;
        let inv_h = 1.0 / h;

        let u_start =
            vertex1.u * (1.0 - (x0 - vertex1.x) * inv_w) + vertex2.u * (x0 - vertex1.x) * inv_w;
        let mut v =
            vertex1.v * (1.0 - (y0 - vertex1.y) * inv_h) + vertex2.v * (y0 - vertex1.y) * inv_h;

        let step_x_u = (vertex2.u - vertex1.u) * inv_w;
        let step_y_v = (vertex2.v - vertex1.v) * inv_h;

        for y in y0 as u16..y1 as u16 {
            let mut u = u_start;
            for x in x0 as u16..x1 as u16 {
                self.render_pixel(
                    x,
                    y,
                    vertex2.z as u32,
                    Rgba {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: color.a,
                    },
                    Uv {
                        u: Fix::from(u),
                        v: Fix::from(v),
                    },
                );
                u += step_x_u;
            }
            v += step_y_v;
        }
    }

    pub fn render_pixel(&mut self, x: u16, y: u16, z: u32, color: Rgba, uv: Uv) {
        // println!("Render pixel: ({x}, {y}) color={color:?} uv={uv:?}");
        // TODO wrap
        // TODo alpha test
        // TODO scan mask

        let z_buffer = self.contextual_registers().z_buffer_settings;
        let frame = self.contextual_registers().frame_buffer_settings;

        match self.contextual_registers().pixel_test.depth_test {
            DepthTest::Never => return,
            DepthTest::Always => {}
            DepthTest::GreaterOrEqual => match z_buffer.storage_format {
                ZStorageFormat::Z32 => {
                    let value = self.read_psmct32(z_buffer.base_pointer, x, y, frame.width);
                    if z < value {
                        return;
                    }
                }
                _ => todo!(),
            },
            DepthTest::Greater => match z_buffer.storage_format {
                ZStorageFormat::Z32 => {
                    let value = self.read_psmct32(z_buffer.base_pointer, x, y, frame.width);
                    if z <= value {
                        return;
                    }
                }
                _ => todo!(),
            },
        };

        if !z_buffer.no_update {
            match z_buffer.storage_format {
                ZStorageFormat::Z32 => {
                    self.write_psmct32(z_buffer.base_pointer, x, y, frame.width, z);
                }
                _ => todo!(),
            }
        }

        let primitive = self.registers.primitive;
        let color = if primitive.texture_mapping {
            let uv = match primitive.texture_coordinate_method {
                TextureCoordinateMethod::Stq => todo!(),
                TextureCoordinateMethod::Uv => uv,
            };

            let texture = self.contextual_registers().texture;
            let texture_color = match texture.pixel_storage_format {
                PixelStorageFormat::Ct32 => Rgba::from(self.read_psmct32(
                    texture.base_pointer,
                    uv.u.round(),
                    uv.v.round(),
                    texture.buffer_width,
                )),
                _ => todo!(),
            };

            match texture.function {
                TextureFunction::Modulate => todo!(),
                TextureFunction::Decal => texture_color,
                TextureFunction::Highlight => todo!(),
                TextureFunction::Highlight2 => todo!(),
            }
        } else {
            color
        };

        let destination = match frame.pixel_storage_format {
            PixelStorageFormat::Ct32 => {
                Rgba::from(self.read_psmct32(frame.base_pointer, x, y, frame.width))
            }
            _ => todo!(),
        };

        let color = if primitive.alpha_blending {
            let alpha = self.contextual_registers().alpha;
            let a = match alpha.input_color_a {
                InputColor::Source => color,
                InputColor::Destination => destination,
                InputColor::Zero => Rgba::default(),
                InputColor::Reserved => todo!(),
            };
            let b = match alpha.input_color_b {
                InputColor::Source => color,
                InputColor::Destination => destination,
                InputColor::Zero => Rgba::default(),
                InputColor::Reserved => todo!(),
            };
            let c = match alpha.input_alpha_c {
                InputAlpha::Source => color.a,
                InputAlpha::Destination => destination.a,
                InputAlpha::Fixed => alpha.fixed,
                InputAlpha::Reserved => todo!(),
            };
            let d = match alpha.input_color_d {
                InputColor::Source => color,
                InputColor::Destination => destination,
                InputColor::Zero => Rgba::default(),
                InputColor::Reserved => todo!(),
            };

            let r = (((a.r as i32 - b.r as i32) * c as i32) >> 7) + d.r as i32;
            let g = (((a.g as i32 - b.g as i32) * c as i32) >> 7) + d.g as i32;
            let b = (((a.b as i32 - b.b as i32) * c as i32) >> 7) + d.b as i32;
            let (r, g, b) = match self.registers.color_clamp {
                ColorClamp::Mask => (r.bits(0..8), g.bits(0..8), b.bits(0..8)),
                ColorClamp::Clamp => (r.clamp(0, 255), g.clamp(0, 255), b.clamp(0, 255)),
            };
            let a = color.a;
            u32::from_bytes(&[r as u8, g as u8, b as u8, a])
        } else {
            u32::from_bytes(&[color.r, color.g, color.b, color.a])
        };

        let color = color & !frame.drawing_mask
            | u32::from_bytes(&[destination.r, destination.g, destination.b, destination.a])
                & frame.drawing_mask;

        match frame.pixel_storage_format {
            PixelStorageFormat::Ct32 => {
                self.write_psmct32(frame.base_pointer, x, y, frame.width, color)
            }
            _ => todo!(),
        }
    }
}
