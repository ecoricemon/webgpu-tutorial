use super::{Matrix4f, Vector};

pub fn translate_x(d: f32) -> Matrix4f {
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, d, 0.0, 0.0, 1.0,
    ])
}

pub fn translate_y(d: f32) -> Matrix4f {
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, d, 0.0, 1.0,
    ])
}

pub fn translate_z(d: f32) -> Matrix4f {
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, d, 1.0,
    ])
}

pub fn translate(dx: f32, dy: f32, dz: f32) -> Matrix4f {
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, dx, dy, dz, 1.0,
    ])
}

pub fn rotate_x(theta: f32) -> Matrix4f {
    let (sin, cos) = theta.sin_cos();
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, cos, sin, 0.0, 0.0, -sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub fn rotate_y(theta: f32) -> Matrix4f {
    let (sin, cos) = theta.sin_cos();
    Matrix4f::new([
        cos, 0.0, -sin, 0.0, 0.0, 1.0, 0.0, 0.0, sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub fn rotate_z(theta: f32) -> Matrix4f {
    let (sin, cos) = theta.sin_cos();
    Matrix4f::new([
        cos, sin, 0.0, 0.0, -sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub fn rotate_axis(axis: Vector<f32, 3>, theta: f32) -> Matrix4f {
    debug_assert!((axis.norm_l2() - 1.0) < 1e-6);
    let (sin, cos) = theta.sin_cos();
    let o_cos = 1.0 - cos;
    let xy = axis.x() * axis.y();
    let xz = axis.x() * axis.z();
    let yz = axis.y() * axis.z();
    Matrix4f::new([
        // Column 0
        axis.x() * axis.x() * o_cos + cos,
        xy * o_cos + axis.z() * sin,
        xz * o_cos - axis.y() * sin,
        0.0,
        // Column 1
        xy * o_cos - axis.z() * sin,
        axis.y() * axis.y() * o_cos + cos,
        yz * o_cos + axis.x() * sin,
        0.0,
        // Column 2
        xz * o_cos + axis.y() * sin,
        yz * o_cos - axis.x() * sin,
        axis.z() * axis.z() * o_cos + cos,
        0.0,
        // Column 3
        0.0,
        0.0,
        0.0,
        1.0,
    ])
}

pub fn scale_x(f: f32) -> Matrix4f {
    Matrix4f::new([
        f, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub fn scale_y(f: f32) -> Matrix4f {
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, f, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub fn scale_z(f: f32) -> Matrix4f {
    Matrix4f::new([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, f, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub fn scale(fx: f32, fy: f32, fz: f32) -> Matrix4f {
    Matrix4f::new([
        fx, 0.0, 0.0, 0.0, 0.0, fy, 0.0, 0.0, 0.0, 0.0, fz, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    #[rustfmt::skip]
    fn translate_matrix_is_ok() {
        let (dx, dy, dz) = (1.0, 2.0, 3.0);
        assert_eq!(
            translate(dx, dy, dz),
            Matrix4f::new([   // Row-major
                1.0, 0.0, 0.0, dx,
                0.0, 1.0, 0.0, dy,
                0.0, 0.0, 1.0, dz,
                0.0, 0.0, 0.0, 1.0
            ]).transpose()
        );
    }
}
