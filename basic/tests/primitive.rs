use basic::primitive::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
#[rustfmt::skip]
fn mat4f_identity() {
    assert_eq!(
        Mat4f::identity(),
        [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
    )
}

#[wasm_bindgen_test]
fn normalize_arbitrary() {
    let v: Vec4<f32> = [0.1, 0.2, 0.3, 1.0];
    let l: f32 = v.iter().take(3).map(|x| x * x).sum::<f32>().sqrt();
    assert_eq!(v.normalize(), [v[0] / l, v[1] / l, v[2] / l, 1.0]);
}

mod transform {
    use basic::primitive::{transform::*, *};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    #[rustfmt::skip]
    fn transpose_arbitrary() {
        let (dx, dy, dz) = (1.0, 2.0, 3.0);
        assert_eq!(
            translate(dx, dy, dz),
            [   // Row-major
                1.0, 0.0, 0.0, dx,
                0.0, 1.0, 0.0, dy,
                0.0, 0.0, 1.0, dz,
                0.0, 0.0, 0.0, 1.0
            ].transpose()
        );
    }
}
