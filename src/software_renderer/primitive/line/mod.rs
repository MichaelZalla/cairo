use crate::{
    color::{self, Color},
    geometry::intersect::intersect_line_segment_plane,
    graphics::Graphics,
    render::Renderer,
    software_renderer::SoftwareRenderer,
    vec::vec3::{self, Vec3},
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_line(
        &mut self,
        mut start_world_space: Vec3,
        mut end_world_space: Vec3,
        color: Color,
    ) {
        // Clip the line segment against all 6 planes of the frustum.

        for plane in self.clipping_frustum.get_planes() {
            if let Some((_alpha, intersection_point)) =
                intersect_line_segment_plane(plane, start_world_space, end_world_space)
            {
                let start_world_space_clipped = intersection_point;

                let end_world_space_clipped = if plane.get_signed_distance(&start_world_space) > 0.0
                {
                    start_world_space
                } else {
                    end_world_space
                };

                start_world_space = start_world_space_clipped;
                end_world_space = end_world_space_clipped;
            }
        }

        let (start_ndc_space, end_ndc_space) = {
            let shader_context = self.shader_context.borrow();

            (
                shader_context.to_ndc_space(start_world_space),
                shader_context.to_ndc_space(end_world_space),
            )
        };

        self.render_line_from_ndc_space_vecs(&start_ndc_space, &end_ndc_space, color);
    }

    pub(in crate::software_renderer) fn render_line_loop(
        &mut self,
        positions_world_space: &[Vec3],
        first: usize,
        last: usize,
        color: Color,
    ) {
        for i in first..last + 1 {
            let j = if i < last { i + 1 } else { first };

            self.render_line(positions_world_space[i], positions_world_space[j], color);
        }
    }

    pub(in crate::software_renderer) fn _render_axes(
        &mut self,
        position: Option<Vec3>,
        scale: Option<f32>,
    ) {
        let p = position.unwrap_or_default();

        let s = scale.unwrap_or(1.0);

        // X-axis (red)

        self.render_line(p, p + vec3::RIGHT * s, color::RED);

        // Y-axis (blue)

        self.render_line(p, p + vec3::UP * s, color::BLUE);

        // Z-axis (green)

        self.render_line(p, p + vec3::FORWARD * s, color::GREEN);
    }

    pub(in crate::software_renderer) fn _render_ground_plane(&mut self, scale: f32) {
        let parallels_color = color::DARK_GRAY;

        let ten_scaled = 10.0 * scale;

        for i in -10..10 + 1 {
            if i == 0 {
                continue;
            }

            // X-axis parallels

            self.render_line(
                Vec3 {
                    x: -ten_scaled,
                    z: (i as f32 * scale),
                    ..Default::default()
                },
                Vec3 {
                    x: ten_scaled,
                    z: (i as f32 * scale),
                    ..Default::default()
                },
                parallels_color,
            );

            // Z-axis parallels

            self.render_line(
                Vec3 {
                    x: (i as f32 * scale),
                    z: -ten_scaled,
                    ..Default::default()
                },
                Vec3 {
                    x: (i as f32 * scale),
                    z: ten_scaled,
                    ..Default::default()
                },
                parallels_color,
            );
        }

        // X-axis

        self.render_line(
            Vec3 {
                x: -ten_scaled,
                ..Default::default()
            },
            Vec3 {
                x: ten_scaled,
                ..Default::default()
            },
            color::RED,
        );

        // Z-axis

        self.render_line(
            Vec3 {
                z: -ten_scaled,
                ..Default::default()
            },
            Vec3 {
                z: ten_scaled,
                ..Default::default()
            },
            color::GREEN,
        );
    }

    fn render_line_from_ndc_space_vecs(&mut self, start: &Vec3, end: &Vec3, color: Color) {
        // Cull lines that are completely in front of our near plane
        // (z1 <= 0 and z2 <= 0).

        if start.z <= 0.0 && end.z <= 0.0 {
            return;
        }

        match &self.framebuffer {
            Some(rc) => {
                let framebuffer = rc.borrow_mut();

                let color_u32 = color.to_u32();

                match &framebuffer.attachments.forward_ldr {
                    Some(forward_ldr_rc) => {
                        let mut forward_buffer = forward_ldr_rc.borrow_mut();

                        Graphics::line(
                            &mut forward_buffer,
                            (start.x * self.viewport.width as f32) as i32,
                            (start.y * self.viewport.height as f32) as i32,
                            (end.x * self.viewport.width as f32) as i32,
                            (end.y * self.viewport.height as f32) as i32,
                            color_u32,
                        );
                    },
                    None => panic!("Called SoftwareRenderer::render_line_from_ndc_space_vecs() with no forward (LDR) framebuffer attachment!"),
                }
            }
            None => panic!(
                "Called SoftwareRenderer::render_line_from_ndc_space_vecs() with no bound framebuffer!"
            ),
        }
    }
}
