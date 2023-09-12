pub mod color {
    use super::super::{into_vec4, Vec4};
    pub const BLACK: Vec4<u8> = into_vec4(0x00, 0x00, 0x00, 0xFF);
    pub const WHITE: Vec4<u8> = into_vec4(0xFF, 0xFF, 0xFF, 0xFF);
    pub const RED: Vec4<u8> = into_vec4(0xFF, 0x00, 0x00, 0xFF);
    pub const GREEN: Vec4<u8> = into_vec4(0x00, 0xFF, 0x00, 0xFF);
    pub const BLUE: Vec4<u8> = into_vec4(0x00, 0x00, 0xFF, 0xFF);
    pub const YELLOW: Vec4<u8> = into_vec4(0xFF, 0xFF, 0x00, 0xFF);
    pub const MAGENTA: Vec4<u8> = into_vec4(0xFF, 0x00, 0xFF, 0xFF);
    pub const CYAN: Vec4<u8> = into_vec4(0x00, 0xFF, 0xFF, 0xFF);
    pub const GRAY: Vec4<u8> = into_vec4(0x80, 0x80, 0x80, 0xFF);
    pub const HOTPINK: Vec4<u8> = into_vec4(0xFF, 0x69, 0xB4, 0xFF);
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
