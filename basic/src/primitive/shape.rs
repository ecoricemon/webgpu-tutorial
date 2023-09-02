use super::Vertex;

#[allow(unused)]
pub fn make_square(tl: Vertex, bl: Vertex, tr: Vertex, br: Vertex) -> (Vec<Vertex>, Vec<u32>) {
    (vec![tl, bl, tr, br], vec![0, 1, 2, 2, 1, 3])
}

#[allow(unused)]
pub fn make_circle() -> (Vec<Vertex>, Vec<u32>) {
    unimplemented!()
}

#[allow(unused)]
pub fn make_cube() -> (Vec<Vertex>, Vec<u32>) {
    unimplemented!()
}

#[allow(unused)]
pub fn make_sphere() -> (Vec<Vertex>, Vec<u32>) {
    unimplemented!()
}
