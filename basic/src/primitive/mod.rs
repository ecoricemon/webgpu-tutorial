use std::ops;
use std::process::Output;

pub mod shape;
pub mod transform;

pub type Vec4<T> = [T; 4];

trait Number {
    type Output;
    fn zero() -> Self::Output;
    fn one() -> Self::Output;
    fn _sqrt(self) -> Self::Output;
}

impl Number for u8 {
    type Output = u8;
    #[inline(always)]
    fn zero() -> Self::Output {
        0
    }
    #[inline(always)]
    fn one() -> Self::Output {
        1
    }
    #[inline(always)]
    fn _sqrt(self) -> Self::Output {
        panic!("Oops! There's no sqrt() for u8")
    }
}

impl Number for f32 {
    type Output = f32;
    #[inline(always)]
    fn zero() -> Self::Output {
        0.0
    }
    #[inline(always)]
    fn one() -> Self::Output {
        1.0
    }
    #[inline(always)]
    fn _sqrt(self) -> Self::Output {
        self.sqrt()
    }
}

pub trait Vec4Ext<T> {
    fn new() -> Vec4<T>;
    fn _from(value: &[T]) -> Vec4<T>;
    fn add(&self, rhs: &Vec4<T>) -> Vec4<T>;
    fn sub(&self, rhs: &Vec4<T>) -> Vec4<T>;
    fn mul(&self, scalar: T) -> Vec4<T>;
    fn normalize(self) -> Vec4<T>;
}

impl<T> Vec4Ext<T> for Vec4<T>
where
    T: ops::Add<Output = T>
        + ops::Sub<Output = T>
        + ops::Mul<Output = T>
        + ops::DivAssign<T>
        + Number<Output = T>
        + Copy,
{
    fn new() -> Vec4<T> {
        [T::zero(), T::zero(), T::zero(), T::one()]
    }

    fn _from(value: &[T]) -> Vec4<T> {
        match value.len() {
            2 => [value[0], value[1], T::zero(), T::one()],
            3 => [value[0], value[1], value[2], T::one()],
            4 => [value[0], value[1], value[2], value[3]],
            _ => panic!("Inappropriate length of Vec4"),
        }
    }

    #[inline]
    fn add(&self, rhs: &Vec4<T>) -> Vec4<T> {
        [
            self[0] + rhs[0],
            self[1] + rhs[1],
            self[2] + rhs[2],
            self[3],
        ]
    }

    #[inline]
    fn sub(&self, rhs: &Vec4<T>) -> Vec4<T> {
        [
            self[0] - rhs[0],
            self[1] - rhs[1],
            self[2] - rhs[2],
            self[3],
        ]
    }

    #[inline]
    fn mul(&self, scalar: T) -> Vec4<T> {
        [
            self[0] * scalar,
            self[1] * scalar,
            self[2] * scalar,
            self[3],
        ]
    }

    #[inline]
    fn normalize(self) -> Vec4<T> {
        let norm = self
            .iter()
            .take(3)
            .fold(T::zero(), |acc, &x| acc + x * x)
            ._sqrt();
        let mut res = self;
        for x in res.iter_mut().take(3) {
            *x /= norm;
        }
        res
    }
}

/// Column major 4x4 f32 matrix.
pub type Mat4f = [f32; 16];

pub trait Mat4fExt {
    fn identity() -> Mat4f;
    fn transpose(self) -> Mat4f;
    fn mul_v4(self, rhs: Vec4<f32>) -> Vec4<f32>;
    fn mul_m4(self, rhs: Mat4f) -> Mat4f;
}

impl Mat4fExt for Mat4f {
    fn identity() -> Mat4f {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    #[inline]
    fn transpose(self) -> Mat4f {
        [
            self[0], self[4], self[8], self[12], self[1], self[5], self[9], self[13], self[2],
            self[6], self[10], self[14], self[3], self[7], self[11], self[15],
        ]
    }

    #[inline]
    fn mul_v4(self, rhs: Vec4<f32>) -> Vec4<f32> {
        [
            self[0] * rhs[0] + self[4] * rhs[1] + self[8] * rhs[2] + self[12] * rhs[3],
            self[1] * rhs[0] + self[5] * rhs[1] + self[9] * rhs[2] + self[13] * rhs[3],
            self[2] * rhs[0] + self[6] * rhs[1] + self[10] * rhs[2] + self[14] * rhs[3],
            self[3] * rhs[0] + self[7] * rhs[1] + self[11] * rhs[2] + self[15] * rhs[3],
        ]
    }

    #[inline]
    fn mul_m4(self, rhs: Mat4f) -> Mat4f {
        [
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
        ]
    }
}

type PositionType = f32;
const POSITION_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x4;
type ColorType = u8;
const COLOR_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Unorm8x4;
const COLOR_MAX: ColorType = 255;
type NormalType = f32;
const NORMAL_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x4;

type Position = Vec4<PositionType>;
type Color = Vec4<ColorType>;
type Normal = Vec4<NormalType>;

pub trait Random {
    type Output;
    fn random() -> Self::Output;
}

impl Random for Color {
    type Output = Self;
    fn random() -> Self::Output {
        let gen = || (js_sys::Math::random() * (COLOR_MAX as f64)) as ColorType;
        [gen(), gen(), gen(), 1]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: Position,
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
                    format: POSITION_FORMAT,
                },
                wgpu::VertexAttribute {
                    // color
                    offset: size_of::<Position>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: COLOR_FORMAT,
                },
                wgpu::VertexAttribute {
                    // normal
                    offset: (size_of::<Position>() + size_of::<Color>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: NORMAL_FORMAT,
                },
            ],
        }
    }

    pub fn new(pos: Position, color: Color, normal: Option<Normal>) -> Self {
        Self {
            pos,
            color,
            normal: normal.unwrap_or(Vec4::new()),
        }
    }

    pub fn normalize(mut self) -> Self {
        self.pos = self.pos.normalize();
        self.normal = self.normal.normalize();
        self
    }
}

impl<'a, 'b> ops::Add<&'b Vertex> for &'a Vertex {
    type Output = Vertex;
    fn add(self, rhs: &'b Vertex) -> Self::Output {
        Vertex {
            pos: self.pos.add(&rhs.pos),
            color: self.color.add(&rhs.color),
            normal: self.normal.add(&rhs.normal),
        }
    }
}

impl<'a, 'b> ops::Sub<&'b Vertex> for &'a Vertex {
    type Output = Vertex;
    fn sub(self, rhs: &'b Vertex) -> Self::Output {
        Vertex {
            pos: self.pos.sub(&rhs.pos),
            color: self.color.sub(&rhs.color),
            normal: self.normal.sub(&rhs.normal),
        }
    }
}
