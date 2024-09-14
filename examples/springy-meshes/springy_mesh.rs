use crate::{point::Point, strut::Strut};

// #[derive(Default, Debug, Copy, Clone)]
// pub struct Face {
//     pub struts: [usize; 3],
//     pub vertex_angles: [f32; 3],
// }

#[derive(Default, Debug, Clone)]
pub struct SpringyMesh {
    pub points: Vec<Point>,
    pub struts: Vec<Strut>,
    // pub faces: Vec<Face>,
}
