use crate::{point::Point, strut::Strut};

static STRENGTH_PER_UNIT_LENGTH: f32 = 100.0;
static DAMPER_PER_UNIT_LENGTH: f32 = 100.0;

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

impl SpringyMesh {
    pub fn new(points: Vec<Point>, struts: Vec<(usize, usize)>) -> Self {
        let struts: Vec<Strut> = struts
            .into_iter()
            .map(|(i, j)| {
                let rest_length = (points[j].position - points[i].position).mag();

                Strut {
                    points: (i, j),
                    rest_length,
                    strength: STRENGTH_PER_UNIT_LENGTH / (rest_length / 1.0),
                    damper: DAMPER_PER_UNIT_LENGTH / (rest_length / 1.0),
                    delta_length: 1.0,
                }
            })
            .collect();

        Self { points, struts }
    }
}
