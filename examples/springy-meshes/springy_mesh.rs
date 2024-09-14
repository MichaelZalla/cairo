use std::f32::consts::PI;

use cairo::vec::vec3::Vec3;

use crate::{point::Point, strut::Strut};

static STRENGTH_PER_UNIT_LENGTH: f32 = 100.0;
static DAMPER_PER_UNIT_LENGTH: f32 = 100.0;

#[derive(Default, Debug, Copy, Clone)]
pub struct Face {
    pub torsional_strength: f32,
    pub torsional_damper: f32,
    pub points: [usize; 3],
    pub rest_angles: [f32; 3],
}

#[derive(Default, Debug, Clone)]
pub struct SpringyMesh {
    pub points: Vec<Point>,
    pub struts: Vec<Strut>,
    pub faces: Vec<Face>,
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

        let faces = vec![];

        Self {
            points,
            struts,
            faces,
        }
    }

    pub fn new_box(center: Vec3, dimension: f32) -> Self {
        let dimension_over_2 = dimension / 2.0;

        let corners = vec![
            // 0 - Top left
            (-dimension_over_2, dimension_over_2),
            // 1 - Top right
            (dimension_over_2, dimension_over_2),
            // 2 - Bottom right
            (dimension_over_2, -dimension_over_2),
            // 3 - Bottom left
            (-dimension_over_2, -dimension_over_2),
        ];

        let points = corners
            .into_iter()
            .map(|c| Point {
                position: Vec3 {
                    x: center.x + c.0,
                    y: center.y + c.1,
                    z: 0.0,
                },
                ..Default::default()
            })
            .collect();

        let struts = vec![
            // 0 - Top
            (0, 1),
            // 1 - Right
            (1, 2),
            // 2 - Bottom
            (3, 2),
            // 3 - Left
            (0, 3),
            // 4 - Cross (top left to bottom right)
            (0, 2),
        ];

        let mut mesh = Self::new(points, struts);

        mesh.faces = vec![
            Face {
                points: [0, 1, 2], // Top-left, top-right, bottom-right.
                rest_angles: [PI / 2.0, PI / 4.0, PI / 4.0],
                torsional_strength: 100.0,
                torsional_damper: 0.2,
            },
            Face {
                points: [0, 3, 2], // Top-left, bottom-left, bottom-right.
                rest_angles: [PI / 2.0, PI / 4.0, PI / 4.0],
                torsional_strength: 100.0,
                torsional_damper: 0.2,
            },
        ];

        mesh
    }
}
