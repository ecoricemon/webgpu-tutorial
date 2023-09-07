use super::Mat4f;

pub fn translate_x(d: f32) -> Mat4f {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, d, 0.0, 0.0, 1.0,
    ]
}

pub fn translate_y(d: f32) -> Mat4f {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, d, 0.0, 1.0,
    ]
}

pub fn translate_z(d: f32) -> Mat4f {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, d, 1.0,
    ]
}

pub fn translate(dx: f32, dy: f32, dz: f32) -> Mat4f {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, dx, dy, dz, 1.0,
    ]
}

pub fn rotate_x(theta: f32) -> Mat4f {
    let cos = theta.cos();
    let sin = theta.sin();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, cos, sin, 0.0, 0.0, -sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn rotate_y(theta: f32) -> Mat4f {
    let cos = theta.cos();
    let sin = theta.sin();
    [
        cos, 0.0, -sin, 0.0, 0.0, 1.0, 0.0, 0.0, sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn rotate_z(theta: f32) -> Mat4f {
    let cos = theta.cos();
    let sin = theta.sin();
    [
        cos, sin, 0.0, 0.0, -sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn scale_x(f: f32) -> Mat4f {
    [
        f, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn scale_y(f: f32) -> Mat4f {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, f, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn scale_z(f: f32) -> Mat4f {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, f, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

pub fn scale(fx: f32, fy: f32, fz: f32) -> Mat4f {
    [
        fx, 0.0, 0.0, 0.0, 0.0, fy, 0.0, 0.0, 0.0, 0.0, fz, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
