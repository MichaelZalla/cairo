use crate::{
    color::{self, Color},
    geometry::intersect::intersect_line_segment_plane,
    graphics::Graphics,
    matrix::Mat4,
    render::Renderer,
    software_renderer::SoftwareRenderer,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
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
            if plane.get_signed_distance(&start_world_space) < 0.0
                && plane.get_signed_distance(&end_world_space) < 0.0
            {
                return;
            }

            if let Some((_alpha, intersection_point)) =
                intersect_line_segment_plane(plane, start_world_space, end_world_space)
            {
                let start_world_space_clipped = intersection_point + plane.normal * 0.001;

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

    pub(in crate::software_renderer) fn _render_axes(&mut self, transform: Option<&Mat4>) {
        for (basis, color) in [
            (vec3::RIGHT, color::RED),
            (vec3::UP, color::BLUE),
            (vec3::FORWARD, color::GREEN),
        ] {
            let mut start = Vec3::default();
            let mut end = start + basis;

            if let Some(mat) = transform {
                start = (Vec4::position(start) * *mat).to_vec3();
                end = (Vec4::position(end) * *mat).to_vec3();
            }

            self.render_line(start, end, color);
        }
    }

    pub(in crate::software_renderer) fn _render_ground_plane(
        &mut self,
        parallels: usize,
        transform: Option<&Mat4>,
    ) {
        let parallels_color = color::DARK_GRAY;

        let parallels_f32 = parallels as f32;

        let xform = match transform {
            Some(m) => *m,
            None => Default::default(),
        };

        let mut start: Vec4;
        let mut end: Vec4;

        for i in -(parallels as i32)..(parallels + 1) as i32 {
            if i == 0 {
                continue;
            }

            // X-axis parallels

            start = Vec4::position(Vec3 {
                x: -parallels_f32,
                z: (i as f32),
                ..Default::default()
            }) * xform;

            end = Vec4::position(Vec3 {
                x: parallels_f32,
                z: (i as f32),
                ..Default::default()
            }) * xform;

            self.render_line(start.into(), end.into(), parallels_color);

            // Z-axis parallels

            start = Vec4::position(Vec3 {
                x: (i as f32),
                z: -parallels_f32,
                ..Default::default()
            }) * xform;

            end = Vec4::position(Vec3 {
                x: (i as f32),
                z: parallels_f32,
                ..Default::default()
            }) * xform;

            self.render_line(start.into(), end.into(), parallels_color);
        }

        // X-axis

        start = Vec4::position(Vec3 {
            x: -parallels_f32,
            ..Default::default()
        }) * xform;

        end = Vec4::position(Vec3 {
            x: parallels_f32,
            ..Default::default()
        }) * xform;

        self.render_line(start.into(), end.into(), color::RED);

        // Z-axis

        start = Vec4::position(Vec3 {
            z: -parallels_f32,
            ..Default::default()
        }) * xform;

        end = Vec4::position(Vec3 {
            z: parallels_f32,
            ..Default::default()
        }) * xform;

        self.render_line(start.into(), end.into(), color::GREEN);
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
