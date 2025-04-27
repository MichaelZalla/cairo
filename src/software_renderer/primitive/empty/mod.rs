use std::f32::consts::{PI, TAU};

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

    fn get_unit_circle_points(&self, divisions: usize, capsule_length: Option<f32>) -> Vec<Vec4> {
        // Defines a unit circle as a set of points on the X-Y-plane.

        let arc_length = TAU / divisions as f32;

        let num_points = divisions + if capsule_length.is_some() { 2 } else { 0 };

        let mut points = vec![Vec4::position(Default::default()); num_points];

        if divisions.rem_euclid(4) == 0 {
            let points_per_quadrant = divisions / 4;

            // Optimized for quadrants.

            let (bottom_quadrants_index_offset, bottom_quadrants_y_offset) =
                if let Some(length) = capsule_length {
                    (1, -length)
                } else {
                    (0, 0.0)
                };

            for index in 0..points_per_quadrant {
                let theta = arc_length * index as f32;

                let (x, y) = (theta.cos(), theta.sin());

                // Top right quadrant
                points[/*0 * points_per_quadrant + */index].x = x;
                points[/*0 * points_per_quadrant + */index].y = y;

                // Top left quadrant
                points[/*1 * */points_per_quadrant + index].x = -y;
                points[/*1 * */points_per_quadrant + index].y = x;

                // Bottom left quadrant
                points[2 * points_per_quadrant + index + bottom_quadrants_index_offset].x = -x;
                points[2 * points_per_quadrant + index + bottom_quadrants_index_offset].y =
                    -y + bottom_quadrants_y_offset;

                // Bottom right quadrant
                points[3 * points_per_quadrant + index + bottom_quadrants_index_offset].x = y;
                points[3 * points_per_quadrant + index + bottom_quadrants_index_offset].y =
                    -x + bottom_quadrants_y_offset;
            }

            if capsule_length.is_some() {
                points[points_per_quadrant * 2].x = -points[0].x;
                points[points_per_quadrant * 2].y = points[0].y;

                points[points_per_quadrant * 4 + bottom_quadrants_index_offset].x = points[0].x;
                points[points_per_quadrant * 4 + bottom_quadrants_index_offset].y =
                    points[0].y + bottom_quadrants_y_offset;
            }
        } else {
            for (index, point) in points.iter_mut().enumerate() {
                let theta = arc_length * index as f32;

                point.x = theta.cos();
                point.y = theta.sin();
            }
        }

        points
    }

    fn render_circles(
        &mut self,
        divisions: usize,
        transform: &Mat4,
        local_transforms: &[Mat4],
        colors: &[Color],
        capsule_length: Option<f32>,
    ) {
        assert!(
            divisions >= 3,
            "Called SoftwareRenderer::render_circles() with fewer than 3 divisions!"
        );

        // Renders one or more transformed unit circles in world space.

        let points = self.get_unit_circle_points(divisions, capsule_length);

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
        with_basis_vectors: bool,
        color: Option<Color>,
    ) {
        let color = color.unwrap_or(color::ORANGE);

        if with_basis_vectors {
            let scaled_transform = Mat4::scale_uniform(0.25) * *transform;

            self.render_axes(Some(&scaled_transform));
        }

        match display_kind {
            EmptyDisplayKind::Axes => {
                self.render_axes(Some(transform));
            }
            EmptyDisplayKind::Arrow => {
                let arrow_start = Vec4::position(Default::default());

                let arrow_end = Vec4::position(vec3::FORWARD);

                let arrow_head_left =
                    arrow_end + Vec4::vector((-vec3::RIGHT + -vec3::FORWARD) * 0.2);

                let arrow_head_right =
                    arrow_end + Vec4::vector((vec3::RIGHT + -vec3::FORWARD) * 0.2);

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

                self.render_circles(divisions, transform, &local_transforms, &colors, None);
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

                self.render_circles(divisions, transform, &local_transforms, &colors, None);
            }
            EmptyDisplayKind::Capsule(divisions, segment_length) => {
                assert!(
                    divisions % 4 == 0,
                    "Capsule division must be divisible by 4"
                );

                assert!(divisions >= 8, "Called `SoftwareRenderer::render_empty()` on capsule with fewer than 8 divisions!");

                // Renders unit circles on parallel planes.

                let rotate_x = Mat4::rotation_x(PI / 2.0);

                let translate_y = Mat4::translation(Vec3 {
                    y: segment_length,
                    ..Default::default()
                });

                let local_transforms = [rotate_x, rotate_x * translate_y];

                let colors = [color, color];

                self.render_circles(divisions, transform, &local_transforms, &colors, None);

                // Renders cross-axis circles, extended in the middle by `segment_length` units.

                let rotate_y = Mat4::rotation_y(PI / 2.0);

                let local_transforms = [translate_y, rotate_y * translate_y];

                self.render_circles(
                    divisions,
                    transform,
                    &local_transforms,
                    &colors,
                    Some(segment_length),
                );
            }
        }
    }
}
