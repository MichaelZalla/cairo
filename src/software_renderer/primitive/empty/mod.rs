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
        vec4::{self, Vec4},
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

    fn get_unit_circle_points(&self, divisions: usize) -> Vec<Vec4> {
        // Defines a unit circle as a set of points on the X-Y-plane.

        let arc_length = TAU / divisions as f32;

        let mut points = vec![Vec4::new(Default::default(), 1.0); divisions];

        for (index, point) in points.iter_mut().enumerate() {
            let theta = arc_length * index as f32;

            point.x = theta.cos();
            point.y = theta.sin();
        }

        points
    }

    fn render_circles(
        &mut self,
        divisions: usize,
        transform: &Mat4,
        local_transforms: &[Mat4],
        colors: &[Color],
    ) {
        // Renders one or more transformed unit circles in world space.

        let points = self.get_unit_circle_points(divisions);

        for (local_transform, color) in local_transforms.iter().zip(colors) {
            // Transforms the unit points for this circle.

            let world_transform = *local_transform * *transform;

            let points_world_space: Vec<Vec3> = points
                .iter()
                .map(|p| (*p * world_transform).to_vec3())
                .collect();

            // Renders the transformed points as a line segment loop.

            self.render_line_loop(&points_world_space, 0, points_world_space.len() - 1, *color);
        }
    }

    pub(in crate::software_renderer) fn _render_empty(
        &mut self,
        transform: &Mat4,
        display_kind: EmptyDisplayKind,
        color: Option<Color>,
    ) {
        let color = color.unwrap_or(color::ORANGE);

        match display_kind {
            EmptyDisplayKind::Axes => {
                let world_position = (Vec4::new(Default::default(), 1.0) * *transform).to_vec3();

                self.render_axes(Some(world_position), None);
            }
            EmptyDisplayKind::Arrow => {
                let arrow_start = Vec4::new(Default::default(), 1.0);

                let arrow_end = vec4::FORWARD;

                let arrow_head_left =
                    arrow_end + Vec4::new((-vec3::RIGHT + -vec3::FORWARD) * 0.2, 0.0);

                let arrow_head_right =
                    arrow_end + Vec4::new((vec3::RIGHT + -vec3::FORWARD) * 0.2, 0.0);

                let segments: [(Vec4, Vec4); 3] = [
                    (arrow_start, arrow_end),
                    (arrow_end, arrow_head_left),
                    (arrow_end, arrow_head_right),
                ];

                for (start, end) in segments.iter() {
                    let start_transformed = (*start * *transform).to_vec3();
                    let end_transformed = (*end * *transform).to_vec3();

                    self.render_line(start_transformed, end_transformed, color);
                }
            }
            EmptyDisplayKind::Square => {
                self.render_square(transform, color);
            }
            EmptyDisplayKind::Cube => {
                let aabb = AABB::from_min_max(-vec3::ONES, vec3::ONES);

                self.render_aabb(&aabb, Some(transform), color);
            }
            EmptyDisplayKind::Circle(divisions) => {
                let local_transforms = [Mat4::identity()];

                let colors = [color];

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

                let colors: [Color; 3] = [color, color, color];

                self.render_circles(divisions, transform, &local_transforms, &colors);
            }
        }
    }
}
