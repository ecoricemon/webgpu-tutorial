use super::prelude::*;
use std::ops;

/// Column major 4x4 f32 matrix.
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default, PartialEq)]
#[repr(transparent)]
pub struct Matrix4f([f32; 16]);

impl Matrix4f {
    #[inline]
    pub fn new(value: [f32; 16]) -> Self {
        Self(value)
    }

    #[inline]
    pub fn identity() -> Self {
        Self::new([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[inline]
    #[must_use]
    #[rustfmt::skip]
    pub fn transpose(self) -> Self {
        Self::new([
            self.0[0], self.0[4], self.0[8],  self.0[12],
            self.0[1], self.0[5], self.0[9],  self.0[13],
            self.0[2], self.0[6], self.0[10], self.0[14],
            self.0[3], self.0[7], self.0[11], self.0[15],
        ])
    }
}

impl<'a, 'b> ops::Mul<&'b Matrix4f> for &'a Matrix4f {
    type Output = Matrix4f;

    #[must_use]
    #[rustfmt::skip]
    fn mul(self, rhs: &'b Matrix4f) -> Self::Output {
        Matrix4f::new([
            // Column 0
            self.0[0] * rhs.0[0] + self.0[4] * rhs.0[1] + self.0[8] * rhs.0[2] + self.0[12] * rhs.0[3],
            self.0[1] * rhs.0[0] + self.0[5] * rhs.0[1] + self.0[9] * rhs.0[2] + self.0[13] * rhs.0[3],
            self.0[2] * rhs.0[0] + self.0[6] * rhs.0[1] + self.0[10] * rhs.0[2] + self.0[14] * rhs.0[3],
            self.0[3] * rhs.0[0] + self.0[7] * rhs.0[1] + self.0[11] * rhs.0[2] + self.0[15] * rhs.0[3],
            // Column 1
            self.0[0] * rhs.0[4] + self.0[4] * rhs.0[5] + self.0[8] * rhs.0[6] + self.0[12] * rhs.0[7],
            self.0[1] * rhs.0[4] + self.0[5] * rhs.0[5] + self.0[9] * rhs.0[6] + self.0[13] * rhs.0[7],
            self.0[2] * rhs.0[4] + self.0[6] * rhs.0[5] + self.0[10] * rhs.0[6] + self.0[14] * rhs.0[7],
            self.0[3] * rhs.0[4] + self.0[7] * rhs.0[5] + self.0[11] * rhs.0[6] + self.0[15] * rhs.0[7],
            // Column 2
            self.0[0] * rhs.0[8] + self.0[4] * rhs.0[9] + self.0[8] * rhs.0[10] + self.0[12] * rhs.0[11],
            self.0[1] * rhs.0[8] + self.0[5] * rhs.0[9] + self.0[9] * rhs.0[10] + self.0[13] * rhs.0[11],
            self.0[2] * rhs.0[8] + self.0[6] * rhs.0[9] + self.0[10] * rhs.0[10] + self.0[14] * rhs.0[11],
            self.0[3] * rhs.0[8] + self.0[7] * rhs.0[9] + self.0[11] * rhs.0[10] + self.0[15] * rhs.0[11],
            // Column 3
            self.0[0] * rhs.0[12] + self.0[4] * rhs.0[13] + self.0[8] * rhs.0[14] + self.0[12] * rhs.0[15],
            self.0[1] * rhs.0[12] + self.0[5] * rhs.0[13] + self.0[9] * rhs.0[14] + self.0[13] * rhs.0[15],
            self.0[2] * rhs.0[12] + self.0[6] * rhs.0[13] + self.0[10] * rhs.0[14] + self.0[14] * rhs.0[15],
            self.0[3] * rhs.0[12] + self.0[7] * rhs.0[13] + self.0[11] * rhs.0[14] + self.0[15] * rhs.0[15],
        ])
    }
}

impl<'a> ops::Mul<Vector<f32, 2>> for &Matrix4f {
    type Output = Vector<f32, 2>;

    #[must_use]
    fn mul(self, rhs: Vector<f32, 2>) -> Self::Output {
        Vector::<f32, 2>::new(
            self.0[0] * rhs.x() + self.0[4] * rhs.y() + self.0[12],
            self.0[1] * rhs.x() + self.0[5] * rhs.y() + self.0[13],
        )
    }
}

impl<'a> ops::Mul<Vector<f32, 3>> for &Matrix4f {
    type Output = Vector<f32, 3>;

    #[must_use]
    fn mul(self, rhs: Vector<f32, 3>) -> Self::Output {
        Vector::<f32, 3>::new(
            self.0[0] * rhs.x() + self.0[4] * rhs.y() + self.0[8] * rhs.z() + self.0[12],
            self.0[1] * rhs.x() + self.0[5] * rhs.y() + self.0[9] * rhs.z() + self.0[13],
            self.0[2] * rhs.x() + self.0[6] * rhs.y() + self.0[10] * rhs.z() + self.0[14],
        )
    }
}

impl<'a> ops::Mul<Vector<f32, 4>> for &Matrix4f {
    type Output = Vector<f32, 4>;

    #[must_use]
    fn mul(self, rhs: Vector<f32, 4>) -> Self::Output {
        Vector::<f32, 4>::new(
            self.0[0] * rhs.x() + self.0[4] * rhs.y() + self.0[8] * rhs.z() + self.0[12] * rhs.w(),
            self.0[1] * rhs.x() + self.0[5] * rhs.y() + self.0[9] * rhs.z() + self.0[13] * rhs.w(),
            self.0[2] * rhs.x() + self.0[6] * rhs.y() + self.0[10] * rhs.z() + self.0[14] * rhs.w(),
            self.0[3] * rhs.x() + self.0[7] * rhs.y() + self.0[11] * rhs.z() + self.0[15] * rhs.w(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    #[rustfmt::skip]
    fn test_transpose() {
        assert_eq!(
            Matrix4f::new([
                0.1, 0.2, 0.3, 0.4,
                0.5, 0.6, 0.7, 0.8,
                0.9, 1.0, 1.1, 1.2,
                1.3, 1.4, 1.5, 1.6,
            ]),
            Matrix4f::new([
                0.1, 0.5, 0.9, 1.3,
                0.2, 0.6, 1.0, 1.4,
                0.3, 0.7, 1.1, 1.5,
                0.4, 0.8, 1.2, 1.6,
            ]).transpose()
        )
    }
}
