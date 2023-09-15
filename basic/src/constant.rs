#![allow(unused)]

pub mod color {
    use super::super::{arr_to_color, Color};
    pub const BLACK: Color = arr_to_color([0x00, 0x00, 0x00]);
    pub const WHITE: Color = arr_to_color([0xFF, 0xFF, 0xFF]);
    pub const RED: Color = arr_to_color([0xFF, 0x00, 0x00]);
    pub const GREEN: Color = arr_to_color([0x00, 0xFF, 0x00]);
    pub const BLUE: Color = arr_to_color([0x00, 0x00, 0xFF]);
    pub const YELLOW: Color = arr_to_color([0xFF, 0xFF, 0x00]);
    pub const MAGENTA: Color = arr_to_color([0xFF, 0x00, 0xFF]);
    pub const CYAN: Color = arr_to_color([0x00, 0xFF, 0xFF]);
    pub const GRAY: Color = arr_to_color([0x80, 0x80, 0x80]);
    pub const HOTPINK: Color = arr_to_color([0xFF, 0x69, 0xB4]);
}

pub mod radian {
    // π/5 (36°)
    pub const FRAC_PI_5: f32 = 0.62831853071795864769252867665590057_f32;
    // 2π/5 (72°)
    pub const FRAC_TAU_5: f32 = 1.25663706143591729538505735331180115_f32;
    // π/2 (90°)
    pub const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
    // π/4 (45°)
    pub const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;
    // π (180°)
    pub const PI: f32 = std::f32::consts::PI;
    // 2π (360°)
    pub const TAU: f32 = std::f32::consts::TAU;
}
