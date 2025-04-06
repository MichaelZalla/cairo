use std::f32::consts::TAU;

use crate::{
    color::{self, Color},
    geometry::primitives::aabb::AABB,
    matrix::Mat4,
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

impl SoftwareRenderer {
    fn render_square(&mut self, transform: &Mat4, color: Color) {
        static SQUARE_VERTICES_OBJECT_SPACE: [Vec4; 4] = [
            // Top-left
            Vec4 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
            // Top-right
            Vec4 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
                w: 1.0,
            },
            // Bottom-right
            Vec4 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
                w: 1.0,
            },
            // Bottom-left
            Vec4 {
                x: -1.0,
                y: -1.0,
                z: 0.0,
                w: 1.0,
            },
        ];

        let positions_world_space: [Vec3; 4] =
            SQUARE_VERTICES_OBJECT_SPACE.map(|p| (p * *transform).to_vec3());

        self.render_line_loop(&positions_world_space, 0, 3, color);
    }

    fn render_circles(
        &mut self,
        divisions: usize,
        transform: &Mat4,
        local_transforms: &[Mat4],
        colors: &[Color],
    ) {
        // 1. Defines a unit circle as a set of points (in object space).

        let arc_length = TAU / divisions as f32;

        let mut points = vec![Vec4::new(Default::default(), 1.0); divisions];

        for (index, point) in points.iter_mut().enumerate() {
            let theta = arc_length * index as f32;

            point.x = theta.cos();
            point.y = theta.sin();
        }

        for (local_transform, color) in local_transforms.iter().zip(colors) {
            let mut transformed_points = points.clone();

            // 2. Transforms the points into world space, based on `transform`.

            for point in transformed_points.iter_mut() {
                *point *= (*local_transform) * *transform;
            }

            // 3. Renders the transformed circle as a set of connected line segments;

            for i in 0..transformed_points.len() {
                let start = &transformed_points[i];

                let end = &transformed_points[if i == transformed_points.len() - 1 {
                    0
                } else {
                    i + 1
                }];

                self.render_line(start.to_vec3(), end.to_vec3(), *color);
            }
        }
    }

    pub(in crate::software_renderer) fn _render_empty(
        &mut self,
        transform: &Mat4,
        display_kind: EmptyDisplayKind,
    ) {
        match display_kind {
            EmptyDisplayKind::Axes => {
                let world_position = (Vec4::new(Default::default(), 1.0) * *transform).to_vec3();

                self.render_axes(Some(world_position), None);
            }
            EmptyDisplayKind::Square => {
                self.render_square(transform, color::WHITE);
            }
            EmptyDisplayKind::Cube => {
                let aabb = AABB::from_min_max(-vec3::ONES, vec3::ONES);

                self.render_aabb(&aabb, Some(transform), color::WHITE);
            }
            EmptyDisplayKind::Circle(divisions) => {
                let local_transforms = [Mat4::identity()];

                let colors = [color::WHITE];

                self.render_circles(divisions, transform, &local_transforms, &colors);
            }
            EmptyDisplayKind::Sphere(divisions) => {
                let local_transform_z_plane = Mat4::identity();

                let local_transform_y_plane = *Quaternion::new(vec3::RIGHT, TAU / 4.0).mat();

                let local_transform_x_plane = *Quaternion::new(vec3::UP, TAU / 4.0).mat();

                let local_transforms: [Mat4; 3] = [
                    local_transform_z_plane,
                    local_transform_y_plane,
                    local_transform_x_plane,
                ];

                let colors: [Color; 3] = [color::GREEN, color::BLUE, color::RED];

                self.render_circles(divisions, transform, &local_transforms, &colors);
            }
        }
    }
}
