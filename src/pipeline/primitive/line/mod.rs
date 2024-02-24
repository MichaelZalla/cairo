use crate::{
    color::{self, Color},
    graphics::Graphics,
    pipeline::Pipeline,
    vec::vec3::Vec3,
};

impl<'a> Pipeline<'a> {
    pub fn render_line(&mut self, start_world_space: Vec3, end_world_space: Vec3, color: Color) {
        let shader_context = self.shader_context.borrow();

        let start_ndc_space = shader_context.to_ndc_space(start_world_space);
        let end_ndc_space = shader_context.to_ndc_space(end_world_space);

        self.render_line_from_ndc_space_vertices(&start_ndc_space, &end_ndc_space, color);
    }

    pub fn render_point_indicator(&mut self, position: Vec3, scale: f32) {
        // X-axis (red)

        self.render_line(
            Vec3 {
                x: -1.0 * scale,
                y: 0.0,
                z: 0.0,
            } + position,
            Vec3 {
                x: 1.0 * scale,
                y: 0.0,
                z: 0.0,
            } + position,
            color::RED,
        );

        // Y-axis (blue)

        self.render_line(
            Vec3 {
                x: 0.0,
                y: -1.0 * scale,
                z: 0.0,
            } + position,
            Vec3 {
                x: 0.0,
                y: 1.0 * scale,
                z: 0.0,
            } + position,
            color::BLUE,
        );

        // Z-axis (green)

        self.render_line(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -1.0 * scale,
            } + position,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0 * scale,
            } + position,
            color::GREEN,
        );
    }

    pub fn render_world_axes(&mut self, scale: f32) {
        self.render_point_indicator(Default::default(), scale)
    }

    pub fn render_ground_plane(&mut self, scale: f32) {
        for i in -10..10 + 1 {
            // X-axis parallels

            self.render_line(
                Vec3 {
                    x: -10.0 * scale,
                    z: (i as f32 * scale),
                    ..Default::default()
                },
                Vec3 {
                    x: 10.0 * scale,
                    z: (i as f32 * scale),
                    ..Default::default()
                },
                if i == 0 { color::RED } else { color::WHITE },
            );

            // Z-axis parallels

            self.render_line(
                Vec3 {
                    x: (i as f32 * scale),
                    z: -10.0 * scale,
                    ..Default::default()
                },
                Vec3 {
                    x: (i as f32 * scale),
                    z: 10.0 * scale,
                    ..Default::default()
                },
                if i == 0 { color::GREEN } else { color::WHITE },
            );

            // Y-axis

            self.render_line(
                Vec3 {
                    y: -10.0 * scale,
                    ..Default::default()
                },
                Vec3 {
                    y: 10.0 * scale,
                    ..Default::default()
                },
                color::BLUE,
            );
        }
    }

    fn render_line_from_ndc_space_vertices(&mut self, start: &Vec3, end: &Vec3, color: Color) {
        // Cull lines that are completely in front of our near plane
        // (z1 <= 0 and z2 <= 0).

        if start.z <= 0.0 && end.z <= 0.0 {
            return;
        }

        match self.framebuffer {
            Some(rc) => {
                let framebuffer = rc.borrow_mut();

                match &framebuffer.attachments.forward_ldr {
                    Some(forward_buffer_lock) => {
                        let mut forward_buffer = forward_buffer_lock.borrow_mut();

                        Graphics::line(
                            &mut *forward_buffer,
                            (start.x * self.viewport.width as f32) as i32,
                            (start.y * self.viewport.height as f32) as i32,
                            (end.x * self.viewport.width as f32) as i32,
                            (end.y * self.viewport.height as f32) as i32,
                            color,
                        );
                    },
                    None => panic!("Called Graphics::render_line_from_ndc_space_vertices() with no forward (LDR) framebuffer attachment!"),
                }
            }
            None => panic!(
                "Called Graphics::render_line_from_ndc_space_vertices() with no bound framebuffer!"
            ),
        }
    }
}
