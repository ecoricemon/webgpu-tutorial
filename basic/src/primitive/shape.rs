use super::{transform::*, Color, Mat4f, Mat4fExt, Position, Random, Vec4Ext, Vertex};
use crate::constant::radian::*;

pub fn make_square(bl: Vertex, br: Vertex, tl: Vertex, tr: Vertex) -> (Vec<Vertex>, Vec<u32>) {
    (vec![bl, br, tl, tr], vec![0, 1, 2, 2, 1, 3])
}

pub fn make_circle(center: Vertex, radius: f32, vertices: u32) -> (Vec<Vertex>, Vec<u32>) {
    let n = vertices.max(3);
    let verticies = (0..n)
        .map(|i| TAU / n as f32 * i as f32)
        .map(|theta| {
            let (s, c) = theta.sin_cos();
            Vertex {
                pos: [
                    center.pos[0] + radius * c,
                    center.pos[1] + radius * s,
                    center.pos[2],
                    center.pos[3],
                ],
                ..center
            }
        })
        .collect();
    // CCW, Triangle List
    let indices = (2..n).flat_map(|i| [0, i - 1, i]).collect();
    (verticies, indices)
}

pub fn make_cube(
    center: Position,
    width: f32,
    height: f32,
    depth: f32,
    color: Option<Color>,
) -> (Vec<Vertex>, Vec<u32>) {
    let (hw, hh, hd) = (width / 2.0, height / 2.0, depth / 2.0);
    let fbl = center.add(&[-hw, -hh, hd, 0.0]);
    let positions = (0..8)
        .map(|i| {
            fbl.add(&[
                if i & 1 != 0 { width } else { 0.0 },
                if i & 2 != 0 { height } else { 0.0 },
                if i & 4 != 0 { -depth } else { 0.0 },
                0.0,
            ])
        })
        .collect::<Vec<_>>();
    let normals = [
        [1.0, 0.0, 0.0, 0.0],  // Right
        [-1.0, 0.0, 0.0, 0.0], // Left
        [0.0, 1.0, 0.0, 0.0],  // Top
        [0.0, -1.0, 0.0, 0.0], // Bottom
        [0.0, 0.0, 1.0, 0.0],  // Front
        [0.0, 0.0, -1.0, 0.0], // Rear
    ];
    let planes: [[u32; 4]; 6] = [
        [1, 5, 3, 7],
        [4, 0, 6, 2],
        [2, 3, 6, 7],
        [1, 0, 5, 4],
        [0, 1, 2, 3],
        [5, 4, 7, 6],
    ];
    let vertices = planes
        .iter()
        .enumerate()
        .flat_map(|(ni, plane)| {
            plane.map(|pi| Vertex {
                pos: positions[pi as usize],
                color: color.unwrap_or(Color::random()),
                normal: normals[ni],
            })
        })
        .collect();
    // CCW, Triangle List
    let indices = (0..6)
        .map(|i| i * 4)
        .flat_map(|i| [i, i + 1, i + 2, i + 2, i + 1, i + 3])
        .collect();
    (vertices, indices)
}

fn make_icosahedron(color: Option<Color>, cap: Option<usize>) -> Vec<Vertex> {
    // Reference: https://en.wikipedia.org/wiki/Regular_icosahedron
    let a = 1_f32 / 5_f32.sqrt();
    let mut positions = Vec::with_capacity(cap.unwrap_or(12));
    positions.push([0.0, 1.0, 0.0, 1.0]);
    positions.push([2.0 * a, a, 0.0, 1.0]);
    let (s, c) = FRAC_TAU_5.sin_cos();
    for i in 1..=4 {
        positions.push(rotate_y(FRAC_TAU_5 * i as f32).mul_v4(positions[1]));
    }
    let rot_mat = rotate_y(FRAC_PI_5);
    for i in 1..=5 {
        positions.push(rot_mat.mul_v4(positions[i].sub(&[0.0, 2.0 * a, 0.0, 0.0])));
    }
    positions.push([0.0, -1.0, 0.0, 1.0]);
    positions
        .into_iter()
        .map(|pos| Vertex {
            pos,
            color: color.unwrap_or(Color::random()),
            normal: pos,
        })
        .collect()
}

fn slice_edge(buf: &mut [Vertex], off: usize, len: usize, si: usize, ei: usize) {
    if len == 0 {
        return;
    }
    let half = len / 2;
    buf[off + half] = (&buf[si] + &buf[ei]).normalize();
    slice_edge(buf, off, half, si, off + half);
    slice_edge(buf, off + half + 1, half, off + half, ei);
}

pub fn make_sphere(division: usize, color: Option<Color>) -> (Vec<Vertex>, Vec<u32>) {
    // Vertex order: [Seed(V), Shared(E * (2^d - 1)), Inner(F * sum of 1..=2^d - 2)]
    // where V: # of vertices, E: # of edges, F: # of faces, d: division
    // Shared: edge0(Va, Vb, Vc, ...), edge1, ..., edge29
    let d = division;
    let (v, e, f) = (12, 30, 20);
    let (seed_len, shared_len, inner_len) = (
        v,
        e * ((1 << d) - 1),
        f * ((1 << d) - 2) * (1 + ((1 << d) - 2)) / 2,
    );
    let (seed_off, shared_off, inner_off) = (0, seed_len, shared_len);
    let mut vertices = make_icosahedron(color, Some(seed_len + shared_len + inner_len));

    // Set face info
    #[derive(Copy, Clone)]
    enum E {
        F(usize), // Forward(Shared index)
        R(usize), // Reverse(Shared index)
    }
    #[derive(Copy, Clone)]
    struct Face {
        i: [usize; 3], // Seed index
        e: [E; 3],
    }
    let mut faces = [Face {
        i: [0, 0, 0],
        e: [E::F(0), E::F(0), E::F(0)],
    }; 20];
    for i in 0..20 {
        let (i2, i4) = (i >> 1, i >> 2);
        match i % 4 {
            0 => {
                faces[i].i = [0, i4 + 1, (i4 + 1) % 5 + 1];
                faces[i].e = [E::F(i2), E::F(i2 + 1), E::R((i2 + 2) % 10)];
            }
            1 => {
                faces[i].i = [i4 + 1, i4 + 6, (i4 + 1) % 5 + 1];
                faces[i].e = [E::F(i2 + 10), E::R(i2 + 11), E::R(i2 + 1)];
            }
            2 => {
                faces[i].i = [(i4 + 1) % 5 + 1, i4 + 6, (i4 + 1) % 5 + 6];
                faces[i].e = [E::F(i2 + 10), E::R(i2 + 19), E::R((i2 + 1) % 10 + 10)];
            }
            _ => {
                faces[i].i = [(i4 + 1) % 5 + 6, i4 + 6, 11];
                faces[i].e = [E::F(i2 + 19), E::F(i2 + 20), E::R((i2 + 2) % 10 + 20)];
            }
        }
    }

    for f in faces.iter() {
        for (i, j) in f.e.iter().enumerate().filter_map(|(i, e)| match e {
            E::F(j) => Some((i, j)),
            E::R(_) => None,
        }) {
            slice_edge(
                &mut vertices,
                shared_off + j,
                ((1 << d) - 1),
                f.i[i],
                f.i[(i + 1) % 3],
            );
        }
    }

    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vec4;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn shape_icosahedron() {
        let a = 1.0 / 5_f32.sqrt(); // 1 / 5^0.5
        let b = 2.0 * a; // 2 / 5^0.5
        let c = (1.0 - a) / 2.0; // (1 - 1 / 5^0.5) / 2
        let d = ((1.0 + a) / 2.0).sqrt(); // ((1 + 1 / 5^0.5) / 2)^0.5
        let e  = (-1.0 - a) / 2.0; // (-1 - 1 / 5^0.5) / 2
        let f = c.sqrt(); // ((1 - 1 / 5^0.5) / 2)^0.5
        let eps = 1e-6_f32;
        let expect: &[Vec4<f32>] = &[
            [0.0, 1.0, 0.0, 1.0], [b, a, 0.0, 1.0], [c, a, -d, 1.0],
            [e, a, -f, 1.0], [e, a, f, 1.0], [c, a, d, 1.0],
            [-e, -a, -f, 1.0], [-c, -a, -d, 1.0], [-b, -a, 0.0, 1.0],
            [-c, -a, d, 1.0], [-e, -a, f, 1.0], [0.0, -1.0, 0.0, 1.0]
            ];
        let shape = make_icosahedron(None, None);
        assert_eq!(shape.len(), expect.len());
        assert!(shape
            .iter()
            .zip(expect.iter())
            .map(|(res, exp)| (res.pos, exp))
            .all(|(res, exp)| res.iter().zip(exp.iter()).all(|(x, y)| (x - y).abs() < eps)));
    }
}
