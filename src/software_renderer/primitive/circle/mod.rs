use crate::{
    color::Color,
    graphics::Graphics,
    software_renderer::SoftwareRenderer,
    vec::{vec3::Vec3, vec4::Vec4},
};

fn projection_to_ndc(position_projection_space: Vec4) -> Vec3 {
    let w_inverse = 1.0 / position_projection_space.w;

    let mut position_ndc = position_projection_space.to_vec3();

    position_ndc *= w_inverse;

    position_ndc.x = (position_ndc.x + 1.0) / 2.0;
    position_ndc.y = (-position_ndc.y + 1.0) / 2.0;

    position_ndc
}

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_circle(
        &mut self,
        position: &Vec3,
        radius_world_units: f32,
        color: Color,
    ) {
        // Adds radius to position.x in camera-view space, before transforming
        // position to NDC space.

        let shader_context = self.shader_context.borrow();

        let (start_view_space, start_ndc_space) = {
            let view_space = Vec4::new(*position, 1.0) * shader_context.view_inverse_transform;

            let view_projection_space = view_space * shader_context.projection_transform;

            let ndc_space = projection_to_ndc(view_projection_space);

            (view_space, ndc_space)
        };

        let end_ndc_space = {
            let view_space = Vec4 {
                x: start_view_space.x + radius_world_units,
                y: start_view_space.y,
                z: start_view_space.z,
                w: start_view_space.w,
            };

            let view_projection_space = view_space * shader_context.projection_transform;

            projection_to_ndc(view_projection_space)
        };

        let horizontal_radius_ndc = end_ndc_space.x - start_ndc_space.x;

        self.render_circle_at_ndc_space_position(
            &start_ndc_space,
            horizontal_radius_ndc,
            None,
            Some(color),
        );
    }

    fn render_circle_at_ndc_space_position(
        &self,
        position_ndc_space: &Vec3,
        radius_ndc_space: f32,
        fill: Option<Color>,
        border: Option<Color>,
    ) {
        // Cull lines that are completely in front of our near plane (z1 <= 0
        // and z2 <= 0).

        if position_ndc_space.z <= 0.0 {
            return;
        }

        let fill_u32 = fill.map(|c| c.to_u32());
        let border_u32 = border.map(|c| c.to_u32());

        match self.framebuffer.as_ref() {
            Some(framebuffer_rc) => {
                let framebuffer = framebuffer_rc.borrow_mut();

                match &framebuffer.attachments.forward_ldr {
                    Some(forward_buffer_rc) => {
                        let mut forward_buffer = forward_buffer_rc.borrow_mut();

                        Graphics::circle(
                            &mut forward_buffer,
                            (position_ndc_space.x * self.viewport.width as f32) as i32,
                            (position_ndc_space.y * self.viewport.height as f32) as i32,
                            (radius_ndc_space * self.viewport.width as f32) as u32, fill_u32, border_u32);
                    },
                    None => panic!("Called SoftwareRenderer::render_circle_at_ndc_space_position() with no forward (LDR) framebuffer attachment!"),
                }
            }
            None => panic!(
                "Called SoftwareRenderer::render_circle_at_ndc_space_position() with no bound framebuffer!"
            ),
        }
    }
}
