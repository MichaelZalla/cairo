use crate::vec::vec3::{Vec3, Vec3A};

#[derive(Default, Debug, Copy, Clone)]
pub struct Triangle {
    pub vertices: [usize; 3],
    pub centroid: Vec3A,
}

impl Triangle {
    pub fn new(vertices: [usize; 3], a: Vec3, b: Vec3, c: Vec3) -> Self {
        let centroid = (a + b + c) * 0.33333;

        Self {
            vertices,
            centroid: Vec3A { v: centroid },
        }
    }
}
