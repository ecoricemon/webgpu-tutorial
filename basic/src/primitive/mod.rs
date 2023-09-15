use std::{cmp, ops};
pub mod shape;
pub mod transform;
#[macro_use]
pub mod vector;
use vector::*;

/// Column major 4x4 f32 matrix.
pub type Mat4f = [f32; 16];

pub trait Mat4fExt {
    fn new(v: [f32; 16]) -> Mat4f;
    fn identity() -> Mat4f;
    #[must_use]
    fn transpose(self) -> Mat4f;
    fn mul_v3(self, rhs: Vector<f32, 3>) -> Vector<f32, 3>;
    #[must_use]
    fn mul_m4(self, rhs: Mat4f) -> Mat4f;
}

impl Mat4fExt for Mat4f {
    #[inline(always)]
    fn new(v: [f32; 16]) -> Mat4f {
        v
    }

    #[inline]
    fn identity() -> Mat4f {
        Mat4f::new([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[inline]
    fn transpose(self) -> Mat4f {
        Mat4f::new([
            self[0], self[4], self[8], self[12], self[1], self[5], self[9], self[13], self[2],
            self[6], self[10], self[14], self[3], self[7], self[11], self[15],
        ])
    }

    #[inline]
    fn mul_v3(self, rhs: Vector<f32, 3>) -> Vector<f32, 3> {
        Vector::<f32, 3>::new(
            self[0] * rhs.x() + self[4] * rhs.y() + self[8] * rhs.z() + self[12],
            self[1] * rhs.x() + self[5] * rhs.y() + self[9] * rhs.z() + self[13],
            self[2] * rhs.x() + self[6] * rhs.y() + self[10] * rhs.z() + self[14],
        )
    }

    #[inline]
    fn mul_m4(self, rhs: Mat4f) -> Mat4f {
        Mat4f::new([
            // Column 0
            self[0] * rhs[0] + self[4] * rhs[1] + self[8] * rhs[2] + self[12] * rhs[3],
            self[1] * rhs[0] + self[5] * rhs[1] + self[9] * rhs[2] + self[13] * rhs[3],
            self[2] * rhs[0] + self[6] * rhs[1] + self[10] * rhs[2] + self[14] * rhs[3],
            self[3] * rhs[0] + self[7] * rhs[1] + self[11] * rhs[2] + self[15] * rhs[3],
            // Column 1
            self[0] * rhs[4] + self[4] * rhs[5] + self[8] * rhs[6] + self[12] * rhs[7],
            self[1] * rhs[4] + self[5] * rhs[5] + self[9] * rhs[6] + self[13] * rhs[7],
            self[2] * rhs[4] + self[6] * rhs[5] + self[10] * rhs[6] + self[14] * rhs[7],
            self[3] * rhs[4] + self[7] * rhs[5] + self[11] * rhs[6] + self[15] * rhs[7],
            // Column 2
            self[0] * rhs[8] + self[4] * rhs[9] + self[8] * rhs[10] + self[12] * rhs[11],
            self[1] * rhs[8] + self[5] * rhs[9] + self[9] * rhs[10] + self[13] * rhs[11],
            self[2] * rhs[8] + self[6] * rhs[9] + self[10] * rhs[10] + self[14] * rhs[11],
            self[3] * rhs[8] + self[7] * rhs[9] + self[11] * rhs[10] + self[15] * rhs[11],
            // Column 3
            self[0] * rhs[12] + self[4] * rhs[13] + self[8] * rhs[14] + self[12] * rhs[15],
            self[1] * rhs[12] + self[5] * rhs[13] + self[9] * rhs[14] + self[13] * rhs[15],
            self[2] * rhs[12] + self[6] * rhs[13] + self[10] * rhs[14] + self[14] * rhs[15],
            self[3] * rhs[12] + self[7] * rhs[13] + self[11] * rhs[14] + self[15] * rhs[15],
        ])
    }
}

impl_vector!(3, {x: 0}, {y: 1}, {z: 2});
type PointType = f32;
const POINT_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x3;
type ColorType = u8;
const COLOR_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Unorm8x4;
type NormalType = f32;
const NORMAL_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x3;

pub type Point = Vector<PointType, 3>;
pub type Color = [ColorType; 4];
pub type Normal = Vector<NormalType, 3>;

pub const fn arr_to_color(value: [u8; 3]) -> Color {
    [value[0], value[1], value[2], 1]
}

pub trait Random {
    type Output;
    fn random() -> Self::Output;
}

impl Random for Color {
    type Output = Self;
    fn random() -> Self::Output {
        let gen = || (js_sys::Math::random() * (ColorType::MAX as f64)) as ColorType;
        [gen(), gen(), gen(), 1]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub point: Point,
    pub color: Color,
    pub normal: Normal,
    // pub uv: [f32; 2],
}

impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    // pos
                    offset: 0,
                    shader_location: 0,
                    format: POINT_FORMAT,
                },
                wgpu::VertexAttribute {
                    // color
                    offset: size_of::<Point>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: COLOR_FORMAT,
                },
                wgpu::VertexAttribute {
                    // normal
                    offset: (size_of::<Point>() + size_of::<Color>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: NORMAL_FORMAT,
                },
            ],
        }
    }

    pub fn new(point: Point, color: Color, normal: Option<Normal>) -> Self {
        Self {
            point,
            color,
            normal: normal.unwrap_or(Normal::default()),
        }
    }

    pub fn normalize(&mut self) {
        self.point.normalize();
        self.normal.normalize();
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex::new(Point::default(), Color::default(), None)
    }
}

impl From<Point> for Vertex {
    fn from(value: Point) -> Self {
        Vertex::new(value, Color::default(), None)
    }
}

impl<'a, 'b> ops::Add<&'b Vertex> for &'a Vertex {
    type Output = Vertex;

    fn add(self, rhs: &'b Vertex) -> Self::Output {
        Vertex {
            point: self.point + rhs.point,
            color: self.color,
            normal: self.normal + rhs.normal,
        }
    }
}

impl<'a, 'b> ops::Sub<&'b Vertex> for &'a Vertex {
    type Output = Vertex;

    fn sub(self, rhs: &'b Vertex) -> Self::Output {
        Vertex {
            point: self.point - rhs.point,
            color: self.color,
            normal: self.normal - rhs.normal,
        }
    }
}

impl<'a> ops::Mul<f32> for &'a Vertex {
    type Output = Vertex;

    fn mul(self, rhs: f32) -> Self::Output {
        Vertex {
            point: self.point * rhs,
            color: self.color,
            normal: self.normal * rhs,
        }
    }
}

impl<'a> ops::Div<f32> for &'a Vertex {
    type Output = Vertex;

    fn div(self, rhs: f32) -> Self::Output {
        Vertex {
            point: self.point / rhs,
            color: self.color,
            normal: self.normal / rhs,
        }
    }
}

impl cmp::PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point && self.color == other.color && self.normal == other.normal
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
