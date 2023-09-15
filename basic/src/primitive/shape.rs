use super::{transform::*, Color, Mat4fExt, Normal, Point, Random, Vertex};
use crate::constant::radian::*;

#[derive(Copy, Clone)]
enum Edge {
    F(usize, usize), // Forward(start index, length)
    R(usize, usize), // Reverse(start index of its forwarding edge, length)
}

impl Default for Edge {
    fn default() -> Self {
        Edge::F(0, 0)
    }
}

#[derive(Copy, Clone, Default)]
struct Face {
    v: [usize; 3],
    e: [Edge; 3],
}

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
                point: Point::new(
                    center.point.x() + radius * c,
                    center.point.y() + radius * s,
                    center.point.z(),
                ),
                ..center
            }
        })
        .collect();
    // CCW, Triangle List
    let indices = (2..n).flat_map(|i| [0, i - 1, i]).collect();
    (verticies, indices)
}

pub fn make_cube(
    center: Point,
    width: f32,
    height: f32,
    depth: f32,
    color: Option<Color>,
) -> (Vec<Vertex>, Vec<u32>) {
    let (hw, hh, hd) = (width / 2.0, height / 2.0, depth / 2.0);
    let fbl = center + Point::new(-hw, -hh, hd);
    let points: Vec<Point> = (0..8)
        .map(|i| {
            fbl + Point::new(
                if i & 1 != 0 { width } else { 0.0 },
                if i & 2 != 0 { height } else { 0.0 },
                if i & 4 != 0 { -depth } else { 0.0 },
            )
        })
        .collect();
    let normals = [
        Normal::new(1.0, 0.0, 0.0),  // Right
        Normal::new(-1.0, 0.0, 0.0), // Left
        Normal::new(0.0, 1.0, 0.0),  // Top
        Normal::new(0.0, -1.0, 0.0), // Bottom
        Normal::new(0.0, 0.0, 1.0),  // Front
        Normal::new(0.0, 0.0, -1.0), // Rear
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
                point: points[pi as usize],
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

pub fn make_icosahedron(
    radius: f32,
    color: Option<Color>,
    vertex_cap: Option<usize>,
    index_cap: Option<usize>,
) -> (Vec<Vertex>, Vec<u32>) {
    // Reference: https://en.wikipedia.org/wiki/Regular_icosahedron
    let a = 1_f32 / 5_f32.sqrt() * radius;
    let mut points = Vec::with_capacity(vertex_cap.unwrap_or(12));
    points.push(Point::new(0.0, 1.0 * radius, 0.0));
    points.push(Point::new(2.0 * a, a, 0.0));
    for i in 1..=4 {
        points.push(rotate_y(FRAC_TAU_5 * i as f32).mul_v3(points[1]));
    }
    let rot_mat = rotate_y(FRAC_PI_5);
    for i in 1..=5 {
        points.push(rot_mat.mul_v3(points[i] - Point::new(0.0, 2.0 * a, 0.0)));
    }
    points.push(Point::new(0.0, -1.0 * radius, 0.0));
    let vertices: Vec<Vertex> = points
        .into_iter()
        .map(|point| Vertex {
            point,
            color: color.unwrap_or(Color::random()),
            normal: point,
        })
        .collect();
    // CCW, Triangle List
    let mut indices = Vec::with_capacity(index_cap.unwrap_or(60));
    if index_cap.is_none() {
        indices.resize(60, 0);
        indices[..60].copy_from_slice(&[
            0, 1, 2, 1, 6, 2, 2, 6, 7, 7, 6, 11, 0, 2, 3, 2, 7, 3, 3, 7, 8, 8, 7, 11, 0, 3, 4, 3,
            8, 4, 4, 8, 9, 9, 8, 11, 0, 4, 5, 4, 9, 5, 5, 9, 10, 10, 9, 11, 0, 5, 1, 5, 10, 1, 1,
            10, 6, 6, 10, 11,
        ]);
    }
    (vertices, indices)
}

fn cut_arc_into_pow2(buf: &mut [Vertex], off: usize, len: usize, si: usize, ei: usize) {
    if len == 0 {
        return;
    }
    let half = len / 2;
    buf[off + half] = &buf[si] + &buf[ei];
    buf[off + half].normalize();
    cut_arc_into_pow2(buf, off, half, si, off + half);
    cut_arc_into_pow2(buf, off + half + 1, half, off + half, ei);
}

fn cut_icosahedron_edges(
    buf: &mut Vec<Vertex>,
    shared_off: usize,
    shared_unit: usize,
) -> [Face; 20] {
    // Set face info
    let mut faces = [Face::default(); 20];
    for i in 0..20 {
        let (i2, i4) = (i >> 1, i >> 2);
        match i % 4 {
            0 => {
                faces[i].v = [0, i4 + 1, (i4 + 1) % 5 + 1];
                faces[i].e = [
                    Edge::F(shared_off + i2 * shared_unit, shared_unit),
                    Edge::F(shared_off + (i2 + 1) * shared_unit, shared_unit),
                    Edge::R(shared_off + ((i2 + 2) % 10) * shared_unit, shared_unit),
                ]
            }
            1 => {
                faces[i].v = [i4 + 1, i4 + 6, (i4 + 1) % 5 + 1];
                faces[i].e = [
                    Edge::F(shared_off + (i2 + 10) * shared_unit, shared_unit),
                    Edge::R(shared_off + (i2 + 11) * shared_unit, shared_unit),
                    Edge::R(shared_off + (i2 + 1) * shared_unit, shared_unit),
                ]
            }
            2 => {
                faces[i].v = [(i4 + 1) % 5 + 1, i4 + 6, (i4 + 1) % 5 + 6];
                faces[i].e = [
                    Edge::F(shared_off + (i2 + 10) * shared_unit, shared_unit),
                    Edge::R(shared_off + (i2 + 19) * shared_unit, shared_unit),
                    Edge::R(shared_off + ((i2 + 1) % 10 + 10) * shared_unit, shared_unit),
                ]
            }
            _ => {
                faces[i].v = [(i4 + 1) % 5 + 6, i4 + 6, 11];
                faces[i].e = [
                    Edge::F(shared_off + (i2 + 19) * shared_unit, shared_unit),
                    Edge::F(shared_off + (i2 + 20) * shared_unit, shared_unit),
                    Edge::R(shared_off + ((i2 + 2) % 10 + 20) * shared_unit, shared_unit),
                ]
            }
        }
    }

    // Slice forwarding edges only
    for f in faces.iter() {
        for (i, off, len) in f.e.iter().enumerate().filter_map(|(i, e)| match e {
            Edge::F(j, k) => Some((i, *j, *k)),
            Edge::R(_, _) => None,
        }) {
            cut_arc_into_pow2(buf, off, len, f.v[i], f.v[(i + 1) % 3]);
        }
    }

    faces
}

pub fn make_icosphere(
    radius: f32,
    division: usize,
    color: Option<Color>,
) -> (Vec<Vertex>, Vec<u32>) {
    if division == 0 {
        return make_icosahedron(radius, color, None, None);
    }
    // Vertex order: [Seed(V), Shared(E * (2^d - 1)), Inner(F * sum of 1..=2^d - 2)]
    // where V: # of vertices, E: # of edges, F: # of faces, d: division
    // Shared: edge0(Va, Vb, Vc, ...), edge1, ..., edge29
    const V: usize = 12;
    const E: usize = 30;
    const F: usize = 20;
    let d = division;
    let shared_unit = (1 << d) - 1;
    let inner_unit = ((1 << d) - 2) * (1 + ((1 << d) - 2)) / 2;
    let (seed_len, shared_len, inner_len) = (V, E * shared_unit, F * inner_unit);
    let (shared_off, inner_off) = (seed_len, seed_len + shared_len);
    let vertex_len = seed_len + shared_len + inner_len;
    let index_len = 60 * 4_usize.pow(d as u32);
    let (mut vertices, mut indices) =
        make_icosahedron(1.0, color, Some(vertex_len), Some(index_len));
    vertices.resize(vertex_len, Vertex::default());
    indices.clear();

    // cut edges
    let faces = cut_icosahedron_edges(&mut vertices, shared_off, shared_unit);

    // Divide each face
    fn divide(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        f: Face,
        d: usize,
        off: usize,
        len: usize,
    ) {
        if d == 0 {
            indices.extend(f.v.into_iter().map(|i| i as u32));
            return;
        }
        let unwrap = |e: Edge| match e {
            Edge::F(off, _) | Edge::R(off, _) => off,
        };
        let make_half_edge = |e: Edge, front: bool| match (e, front) {
            (Edge::F(off, len), true) => Edge::F(off, len / 2),
            (Edge::F(off, len), false) => Edge::F(off + len / 2 + 1, len / 2),
            (Edge::R(off, len), false) => Edge::R(off, len / 2),
            (Edge::R(off, len), true) => Edge::R(off + len / 2 + 1, len / 2),
        };
        let half = (1 << (d - 1)) - 1;
        let halfs = [
            unwrap(f.e[0]) + half,
            unwrap(f.e[1]) + half,
            unwrap(f.e[2]) + half,
        ];
        let half_offs = [off, off + half, off + 2 * half];
        let mut faces = [Face::default(); 4];
        for i in 0..3 {
            cut_arc_into_pow2(vertices, half_offs[i], half, halfs[i], halfs[(i + 5) % 3]);
            faces[i] = Face {
                v: [f.v[i], halfs[i], halfs[(i + 5) % 3]],
                e: [
                    make_half_edge(f.e[i], true),
                    Edge::F(half_offs[i], half),
                    make_half_edge(f.e[(i + 5) % 3], false),
                ],
            }
        }
        faces[3] = Face {
            v: halfs,
            e: [
                Edge::R(half_offs[1], half),
                Edge::R(half_offs[2], half),
                Edge::R(half_offs[0], half),
            ],
        };
        let new_off = off + 3 * half;
        let new_len = (len - 3 * half) / 4;
        for i in 0..4 {
            divide(
                vertices,
                indices,
                faces[i],
                d - 1,
                new_off + i * new_len,
                new_len,
            );
        }
    }
    for i in 0..faces.len() {
        divide(
            &mut vertices,
            &mut indices,
            faces[i],
            d,
            inner_off + i * inner_unit,
            inner_unit,
        );
    }

    // Copy points into normals, and then adapt the radius
    for v in vertices.iter_mut() {
        v.normal = v.point;
        v.point *= radius;
    }
    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    const EPS: f32 = 1e-6;

    #[wasm_bindgen_test]
    #[rustfmt::skip]
    fn cut_arc_into_pow2_from_2_to_16() {
        for n in [2, 4, 8, 16] {
            let radius = 1.0;
            let mut buf: Vec<Vertex> = vec![Point::default().into(); n + 1];
            let rot_y = rotate_y(-radian::FRAC_PI_4);
            let s = Point::new(radius, 0.0, 0.0);
            let e = Point::new(0.0, radius, 0.0);
            (buf[0].point, buf[1].point) = (rot_y.mul_v3(s), e);

            let mut expect = buf.clone();
            let theta = radian::FRAC_PI_2 / n as f32;
            for i in 1..n {
                expect[i + 1] = rot_y.mul_m4(rotate_z(theta * i as f32)).mul_v3(s).into();
            }

            cut_arc_into_pow2(&mut buf, 2, n - 1, 0, 1);
            assert!(buf
                .iter()
                .map(|v| v.point)
                .zip(expect.iter().map(|v| v.point))
                .all(|(res, exp)| res.iter().zip(exp.iter()).all(|(x, y)| (x - y).abs() < EPS)));
        }
    }

    #[wasm_bindgen_test]
    fn cut_icosahedron_edges_makes_the_same_sized_pieces() {
        for d in 0..=5 {
            const V: usize = 12;
            const E: usize = 30;
            let shared_unit = (1 << d) - 1;
            let (seed_len, shared_len) = (V, E * shared_unit);
            let shared_off = seed_len;
            let vertex_len = seed_len + shared_len;
            let (mut buf, _) = make_icosahedron(1.0, None, Some(vertex_len), None);

            buf.resize(seed_len + shared_len, Vertex::default());
            let faces = cut_icosahedron_edges(&mut buf, shared_off, shared_unit);
            let (mut low, mut high) = (f32::MAX, f32::MIN);
            for f in faces {
                for i in 0..3 {
                    let mut points: Vec<Point> = vec![buf[f.v[i]].point];
                    match f.e[i] {
                        Edge::F(si, len) => {
                            for j in si..si + len {
                                points.push(buf[j].point);
                            }
                        }
                        Edge::R(si, len) => {
                            for j in (si..si + len).rev() {
                                points.push(buf[j].point);
                            }
                        }
                    }
                    points.push(buf[f.v[(i + 1) % 3]].point);
                    for win in points.windows(2) {
                        let d = win[0].dist(win[1]);
                        low = low.min(d);
                        high = high.max(d);
                    }
                }
            }
            assert!(high - low < EPS);
        }
    }
}
