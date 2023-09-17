mod constant;
mod shape;
mod vertex;

pub use constant::*;
pub use shape::*;
pub use vertex::*;

pub mod prelude {
    pub use crate::{constant::*, shape::*, vertex::*, Color, Normal, Point, Vertex};
}

// Define Point, Color, Normal, and Vertex
define_vertex!([f32; 3], [u8; 4], [f32; 3]);

// constant setter
const fn u8x4_to_color(value: [u8; 4]) -> Color {
    my_math::Vector(value)
}
