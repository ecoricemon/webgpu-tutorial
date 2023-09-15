use std::{cmp, ops};
pub mod shape;
pub mod transform;
#[macro_use]
pub mod vector;
use vector::*;
pub mod matrix;
use matrix::*;

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
