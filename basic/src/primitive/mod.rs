use std::process::Output;
use std::{cmp, ops};

pub mod shape;
pub mod transform;

trait Number {
    type Output;
    fn zero() -> Self::Output;
    fn one() -> Self::Output;
    fn _sqrt(self) -> Self::Output;
    fn _acos(self) -> Self::Output;
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

    #[inline(always)]
    fn _acos(self) -> Self::Output {
        panic!("Oops! There's no acos() for u8")
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

    #[inline(always)]
    fn _acos(self) -> Self::Output {
        self.acos()
    }
}

pub type Vec4<T> = [T; 4];

pub trait Vec4Ext<T> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
    fn w(&self) -> T;
    fn set_x(&mut self, v: T);
    fn set_y(&mut self, v: T);
    fn set_z(&mut self, v: T);
    fn set_w(&mut self, v: T);
    fn new3(x: T, y: T, z: T) -> Vec4<T>;
    fn new4(x: T, y: T, z: T, w: T) -> Vec4<T>;
    fn _default() -> Vec4<T>;
    fn _from(value: &[T]) -> Vec4<T>;
    fn add_v4(&self, rhs: &Vec4<T>) -> Vec4<T>;
    fn add_v4_assign(&mut self, rhs: &Vec4<T>);
    fn sub_v4(&self, rhs: &Vec4<T>) -> Vec4<T>;
    fn sub_v4_assign(&mut self, rhs: &Vec4<T>);
    fn mul(&self, scalar: T) -> Vec4<T>;
    fn mul_assign(&mut self, scalar: T);
    fn div(&self, scalar: T) -> Vec4<T>;
    fn div_assign(&mut self, scalar: T);
    fn norm_l2(&self) -> T;
    fn normalize(self) -> Vec4<T>;
    fn dist(&self, rhs: &Vec4<T>) -> T;
    fn dot_product(&self, rhs: &Vec4<T>) -> T;
    fn cross_product(&self, rhs: &Vec4<T>) -> Vec4<T>;
    fn angle_between(&self, rhs: &Vec4<T>) -> T;
}

impl<T> Vec4Ext<T> for Vec4<T>
where
    T: ops::Add<Output = T>
        + ops::Sub<Output = T>
        + ops::Mul<Output = T>
        + ops::Div<Output = T>
        + PartialEq
        + Number<Output = T>
        + Copy,
{
    #[inline(always)]
    fn x(&self) -> T {
        self[0]
    }

    #[inline(always)]
    fn y(&self) -> T {
        self[1]
    }

    #[inline(always)]
    fn z(&self) -> T {
        self[2]
    }

    #[inline(always)]
    fn w(&self) -> T {
        self[3]
    }

    #[inline(always)]
    fn set_x(&mut self, v: T) {
        self[0] = v;
    }

    #[inline(always)]
    fn set_y(&mut self, v: T) {
        self[1] = v;
    }

    #[inline(always)]
    fn set_z(&mut self, v: T) {
        self[2] = v;
    }

    #[inline(always)]
    fn set_w(&mut self, v: T) {
        self[3] = v;
    }

    #[inline(always)]
    fn new3(x: T, y: T, z: T) -> Vec4<T> {
        [x, y, z, T::one()]
    }

    #[inline(always)]
    fn new4(x: T, y: T, z: T, w: T) -> Vec4<T> {
        [x, y, z, w]
    }

    #[inline(always)]
    fn _default() -> Vec4<T> {
        Vec4::new3(T::zero(), T::zero(), T::zero())
    }

    #[inline]
    fn _from(value: &[T]) -> Vec4<T> {
        match value.len() {
            2 => Vec4::new3(value[0], value[1], T::zero()),
            3 => Vec4::new3(value[0], value[1], value[2]),
            4 => Vec4::new4(value[0], value[1], value[2], value[3]),
            _ => panic!("Inappropriate length of Vec4"),
        }
    }

    #[inline]
    fn add_v4(&self, rhs: &Vec4<T>) -> Vec4<T> {
        Vec4::new4(
            self.x() + rhs.x(),
            self.y() + rhs.y(),
            self.z() + rhs.z(),
            self.w(),
        )
    }

    #[inline]
    fn add_v4_assign(&mut self, rhs: &Vec4<T>) {
        self.set_x(self.x() + rhs.x());
        self.set_y(self.y() + rhs.y());
        self.set_z(self.z() + rhs.z());
    }

    #[inline]
    fn sub_v4(&self, rhs: &Vec4<T>) -> Vec4<T> {
        Vec4::new4(
            self.x() - rhs.x(),
            self.y() - rhs.y(),
            self.z() - rhs.z(),
            self.w(),
        )
    }

    #[inline]
    fn sub_v4_assign(&mut self, rhs: &Vec4<T>) {
        self.set_x(self.x() - rhs.x());
        self.set_y(self.y() - rhs.y());
        self.set_z(self.z() - rhs.z());
    }

    #[inline]
    fn mul(&self, scalar: T) -> Vec4<T> {
        Vec4::new4(
            self.x() * scalar,
            self.y() * scalar,
            self.z() * scalar,
            self.w(),
        )
    }

    #[inline]
    fn mul_assign(&mut self, scalar: T) {
        self.set_x(self.x() * scalar);
        self.set_y(self.y() * scalar);
        self.set_z(self.z() * scalar);
    }

    #[inline]
    fn div(&self, scalar: T) -> Vec4<T> {
        Vec4::new4(
            self.x() / scalar,
            self.y() / scalar,
            self.z() / scalar,
            self.w(),
        )
    }

    #[inline]
    fn div_assign(&mut self, scalar: T) {
        self.set_x(self.x() / scalar);
        self.set_y(self.y() / scalar);
        self.set_z(self.z() / scalar);
    }

    #[inline]
    fn norm_l2(&self) -> T {
        (self.x() * self.x() + self.y() * self.y() + self.z() * self.z())._sqrt()
    }

    #[inline]
    fn normalize(self) -> Vec4<T> {
        let norm = self.norm_l2();
        match norm != T::zero() {
            true => Vec4::new4(self.x() / norm, self.y() / norm, self.z() / norm, self.w()),
            false => self,
        }
    }

    #[inline]
    fn dist(&self, rhs: &Vec4<T>) -> T {
        self.sub_v4(rhs).norm_l2()
    }

    #[inline]
    fn dot_product(&self, rhs: &Vec4<T>) -> T {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z()
    }

    #[inline]
    fn cross_product(&self, rhs: &Vec4<T>) -> Vec4<T> {
        [
            self.y() * rhs.z() - self.z() * rhs.y(),
            self.z() * rhs.x() - self.x() * rhs.z(),
            self.x() * rhs.y() - self.y() * rhs.x(),
            self.w(),
        ]
    }

    #[inline]
    fn angle_between(&self, rhs: &Vec4<T>) -> T {
        (self.dot_product(rhs) / self.norm_l2() / rhs.norm_l2())._acos()
    }
}

pub const fn into_vec4<T>(x: T, y: T, z: T, w: T) -> Vec4<T> {
    [x, y, z, w]
}

/// Column major 4x4 f32 matrix.
pub type Mat4f = [f32; 16];

pub trait Mat4fExt {
    fn new(v: [f32; 16]) -> Mat4f;
    fn identity() -> Mat4f;
    fn transpose(self) -> Mat4f;
    fn mul_v4(self, rhs: &Vec4<f32>) -> Vec4<f32>;
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
    fn mul_v4(self, rhs: &Vec4<f32>) -> Vec4<f32> {
        Vec4::new4(
            self[0] * rhs[0] + self[4] * rhs[1] + self[8] * rhs[2] + self[12] * rhs[3],
            self[1] * rhs[0] + self[5] * rhs[1] + self[9] * rhs[2] + self[13] * rhs[3],
            self[2] * rhs[0] + self[6] * rhs[1] + self[10] * rhs[2] + self[14] * rhs[3],
            self[3] * rhs[0] + self[7] * rhs[1] + self[11] * rhs[2] + self[15] * rhs[3],
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

type PositionType = f32;
const POSITION_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x4;
type ColorType = u8;
const COLOR_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Unorm8x4;
const COLOR_MAX: ColorType = 255;
type NormalType = f32;
const NORMAL_FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32x4;

pub type Position = Vec4<PositionType>;
pub type Color = Vec4<ColorType>;
pub type Normal = Vec4<NormalType>;

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
            normal: normal.unwrap_or(Vec4::default()),
        }
    }

    pub fn normalize(mut self) -> Self {
        self.pos = self.pos.normalize();
        self.normal = self.normal.normalize();
        self
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex::new(Vec4::_default(), Vec4::_default(), None)
    }
}

impl From<Position> for Vertex {
    fn from(value: Position) -> Self {
        Vertex::new(value, Vec4::_default(), None)
    }
}

impl<'a, 'b> ops::Add<&'b Vertex> for &'a Vertex {
    type Output = Vertex;

    fn add(self, rhs: &'b Vertex) -> Self::Output {
        Vertex {
            pos: self.pos.add_v4(&rhs.pos),
            color: self.color,
            normal: self.normal.add_v4(&rhs.normal),
        }
    }
}

impl<'a, 'b> ops::Sub<&'b Vertex> for &'a Vertex {
    type Output = Vertex;

    fn sub(self, rhs: &'b Vertex) -> Self::Output {
        Vertex {
            pos: self.pos.sub_v4(&rhs.pos),
            color: self.color,
            normal: self.normal.sub_v4(&rhs.normal),
        }
    }
}

impl<'a> ops::Div<f32> for &'a Vertex {
    type Output = Vertex;

    fn div(self, rhs: f32) -> Self::Output {
        Vertex {
            pos: self.pos.div(rhs),
            color: self.color,
            normal: self.normal.div(rhs),
        }
    }
}

impl<'a> ops::Mul<f32> for &'a Vertex {
    type Output = Vertex;

    fn mul(self, rhs: f32) -> Self::Output {
        Vertex {
            pos: self.pos.mul(rhs),
            color: self.color,
            normal: self.normal.mul(rhs),
        }
    }
}

impl cmp::PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.color == other.color && self.normal == other.normal
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
