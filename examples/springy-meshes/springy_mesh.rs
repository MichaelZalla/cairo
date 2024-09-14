use std::f32::consts::PI;

use cairo::{
    animation::lerp,
    color::{self, Color},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use crate::{
    coordinates::world_to_screen_space, point::Point, renderable::Renderable, strut::Strut,
};

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

fn draw_line(
    start_world_space: &Vec3,
    end_world_space: &Vec3,
    color: &Color,
    buffer: &mut Buffer2D,
    buffer_center: &Vec3,
) {
    let start_screen_space = world_to_screen_space(&start_world_space, buffer_center);
    let end_screen_space = world_to_screen_space(&end_world_space, buffer_center);

    let (x1, y1) = (start_screen_space.x as i32, start_screen_space.y as i32);
    let (x2, y2) = (end_screen_space.x as i32, end_screen_space.y as i32);

    Graphics::line(buffer, x1, y1, x2, y2, &color);
}

impl Renderable for SpringyMesh {
    fn render(&self, buffer: &mut cairo::buffer::Buffer2D, buffer_center: &Vec3) {
        // Draw each point (vertex) as a square.

        for point in &self.points {
            point.render(buffer, buffer_center);
        }

        // Draws each strut as a line, using color to indicate its current
        // compression/elongation.
        for strut in &self.struts {
            let start_world_space = &self.points[strut.points.0].position;
            let end_world_space = &self.points[strut.points.1].position;

            let elongation_alpha =
                ((strut.rest_length + strut.delta_length) / strut.rest_length / 2.0)
                    .clamp(0.0, 1.0);

            let color_vec3 = lerp(
                color::RED.to_vec3(),
                color::BLUE.to_vec3(),
                elongation_alpha,
            );

            let color = Color::from_vec3(color_vec3);

            draw_line(
                start_world_space,
                end_world_space,
                &color,
                buffer,
                buffer_center,
            );
        }
    }
}
