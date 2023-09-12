use basic::primitive::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const EPS: f32 = 1e-6;

#[wasm_bindgen_test]
#[rustfmt::skip]
fn transpose_arbitrary() {
    assert_eq!(
        [
            0.1, 0.2, 0.3, 0.4,
            0.5, 0.6, 0.7, 0.8,
            0.9, 1.0, 1.1, 1.2,
            1.3, 1.4, 1.5, 1.6,
        ],
        [
            0.1, 0.5, 0.9, 1.3,
            0.2, 0.6, 1.0, 1.4,
            0.3, 0.7, 1.1, 1.5,
            0.4, 0.8, 1.2, 1.6,
        ].transpose()
    )
}

#[wasm_bindgen_test]
fn normalize_arbitrary() {
    let v: Vec4<f32> = [0.1, 0.2, 0.3, 1.0];
    let l: f32 = v.iter().take(3).map(|x| x * x).sum::<f32>().sqrt();
    assert_eq!(v.normalize(), [v[0] / l, v[1] / l, v[2] / l, 1.0]);
}

mod transform {
    use super::*;
    use basic::primitive::transform::*;

    #[wasm_bindgen_test]
    #[rustfmt::skip]
    fn translate_matrix_is_ok() {
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

mod shape {
    use super::*;
    use basic::primitive::{shape::*, transform::scale};

    #[wasm_bindgen_test]
    fn make_circle_returns_circle() {
        let center: Vertex = Position::new3(1.0, 2.0, 3.0).into();
        let radius = 1.0;
        let vertices = 16;
        let res = make_circle(center, radius, vertices);
        assert_eq!(res.0.len() as u32, vertices);
        assert!(res
            .0
            .iter()
            .map(|v| v.pos.dist(&center.pos))
            .all(|d| (d - radius).abs() < EPS));
    }

    #[wasm_bindgen_test]
    fn make_icosahedron_vertices_are_on_exact_positions() {
        for radius in [0.5, 1.0, 2.0] {
            let a = 1.0 / 5_f32.sqrt(); // 1 / 5^0.5
            let b = 2.0 * a; // 2 / 5^0.5
            let c = (1.0 - a) / 2.0; // (1 - 1 / 5^0.5) / 2
            let d = ((1.0 + a) / 2.0).sqrt(); // ((1 + 1 / 5^0.5) / 2)^0.5
            let e = (-1.0 - a) / 2.0; // (-1 - 1 / 5^0.5) / 2
            let f = c.sqrt(); // ((1 - 1 / 5^0.5) / 2)^0.5
            let mut expect = [
                Position::new3(0.0, 1.0, 0.0),
                Position::new3(b, a, 0.0),
                Position::new3(c, a, -d),
                Position::new3(e, a, -f),
                Position::new3(e, a, f),
                Position::new3(c, a, d),
                Position::new3(-e, -a, -f),
                Position::new3(-c, -a, -d),
                Position::new3(-b, -a, 0.0),
                Position::new3(-c, -a, d),
                Position::new3(-e, -a, f),
                Position::new3(0.0, -1.0, 0.0),
            ];
            for exp_v in expect.iter_mut() {
                *exp_v = scale(radius, radius, radius).mul_v4(exp_v);
            }
            let (vertices, indices) = make_icosahedron(radius, None, None, None);
            assert_eq!(vertices.len(), expect.len());
            assert_eq!(indices.len(), 60);
            assert!(vertices
                .iter()
                .map(|v| v.pos)
                .zip(expect.iter())
                .all(|(res, exp)| res.iter().zip(exp.iter()).all(|(x, y)| (x - y).abs() < EPS)));
        }
    }

    #[wasm_bindgen_test]
    #[rustfmt::skip]
    fn make_icosphere_returns_sphere_with_division_from_0_to_4() {
        for (radius, division, vertex_num, index_num) in [
            (1.0, 0,   12,   20 * 3),
            (1.0, 1,   42,   80 * 3),
            (1.0, 2,  162,  320 * 3),
            (1.0, 3,  642, 1280 * 3),
            (1.0, 4, 2562, 5120 * 3),
            (0.5, 2,  162,  320 * 3),
            (2.0, 2,  162,  320 * 3),
        ] {
            let (vertices, indices) = make_icosphere(radius, division, None);
            assert_eq!(vertices.len(), vertex_num);
            assert_eq!(indices.len(), index_num);
            assert!(vertices
                .iter()
                .map(|v| v.pos)
                .all(|p| (p.norm_l2() - radius).abs() < EPS),
                "radius: {radius}, division: {division}"
            );
        }
    }
}
