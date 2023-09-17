#[macro_export]
macro_rules! define_vertex {
    ([$ptype:ty; $pdim:expr], [$ctype:ty; $cdim:expr], [$ntype:ty; $ndim:expr]) => {
        pub type Point = eg_math::prelude::Vector<$ptype, $pdim>;
        pub type Color = eg_math::prelude::Vector<$ctype, $cdim>;
        pub type Normal = eg_math::prelude::Vector<$ntype, $ndim>;

        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        #[repr(C)]
        pub struct Vertex {
            pub point: Point,
            pub color: Color,
            pub normal: Normal,
            // pub uv: [f32; 2],
        }

        impl Vertex {
            fn get_format(type_name: Option<&str>, dim: usize) -> wgpu::VertexFormat {
                match (type_name, dim) {
                    (Some("u8"), 2) => wgpu::VertexFormat::Unorm8x2,
                    (Some("u8"), 4) => wgpu::VertexFormat::Unorm8x4,
                    (Some("f32"), 1) => wgpu::VertexFormat::Float32,
                    (Some("f32"), 2) => wgpu::VertexFormat::Float32x2,
                    (Some("f32"), 3) => wgpu::VertexFormat::Float32x3,
                    (Some("f32"), 4) => wgpu::VertexFormat::Float32x4,
                    _ => panic!(
                        "Unsupported type '{}' and dimension '{}'",
                        type_name.unwrap_or("None"),
                        dim
                    ),
                }
            }

            pub fn vertex_attribute() -> [wgpu::VertexAttribute; 3] {
                [
                    wgpu::VertexAttribute {
                        // point
                        offset: 0,
                        shader_location: 0,
                        format: Self::get_format(Point::get_type(), Point::get_dim()),
                    },
                    wgpu::VertexAttribute {
                        // color
                        offset: std::mem::size_of::<Point>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: Self::get_format(Color::get_type(), Color::get_dim()),
                    },
                    wgpu::VertexAttribute {
                        // normal
                        offset: (std::mem::size_of::<Point>() + std::mem::size_of::<Color>())
                            as wgpu::BufferAddress,
                        shader_location: 2,
                        format: Self::get_format(Normal::get_type(), Normal::get_dim()),
                    },
                ]
            }

            pub fn layout<'a>(
                attributes: &'a [wgpu::VertexAttribute; 3],
            ) -> wgpu::VertexBufferLayout<'a> {
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                }
            }

            pub fn new(point: Point, color: Color, normal: Normal) -> Self {
                Self {
                    point,
                    color,
                    normal,
                }
            }

            pub fn normalize(&mut self) {
                self.point.normalize();
                self.normal.normalize();
            }

            pub fn make_unit(self) -> Self {
                Self::new(self.point.make_unit(), self.color, self.normal.make_unit())
            }
        }

        impl Default for Vertex {
            fn default() -> Self {
                Vertex::new(Point::default(), Color::default(), Normal::default())
            }
        }

        impl From<Point> for Vertex {
            fn from(point: Point) -> Self {
                Vertex::new(point, Color::default(), Normal::default())
            }
        }

        impl<'a, 'b> std::ops::Add<&'b Vertex> for &'a Vertex {
            type Output = Vertex;

            fn add(self, rhs: &'b Vertex) -> Self::Output {
                Vertex {
                    point: self.point + rhs.point,
                    color: self.color,
                    normal: self.normal + rhs.normal,
                }
            }
        }

        impl<'a, 'b> std::ops::Sub<&'b Vertex> for &'a Vertex {
            type Output = Vertex;

            fn sub(self, rhs: &'b Vertex) -> Self::Output {
                Vertex {
                    point: self.point - rhs.point,
                    color: self.color,
                    normal: self.normal - rhs.normal,
                }
            }
        }

        impl<'a> std::ops::Mul<f32> for &'a Vertex {
            type Output = Vertex;

            fn mul(self, rhs: f32) -> Self::Output {
                Vertex {
                    point: self.point * rhs,
                    color: self.color,
                    normal: self.normal * rhs,
                }
            }
        }

        impl<'a> std::ops::Div<f32> for &'a Vertex {
            type Output = Vertex;

            fn div(self, rhs: f32) -> Self::Output {
                Vertex {
                    point: self.point / rhs,
                    color: self.color,
                    normal: self.normal / rhs,
                }
            }
        }

        impl std::cmp::PartialEq for Vertex {
            fn eq(&self, other: &Self) -> bool {
                self.point == other.point
                    && self.color == other.color
                    && self.normal == other.normal
            }

            fn ne(&self, other: &Self) -> bool {
                !self.eq(other)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    define_vertex!([f32; 3], [u8; 4], [f32; 3]);
}
