use crate::{
    color::Color,
    graphics::{
        Graphics,
        text::{TextOperation, cache::cache_text},
    },
    matrix::Mat4,
    software_renderer::SoftwareRenderer,
    vec::{vec3::Vec3, vec4::Vec4},
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_text(
        &mut self,
        transform: &Mat4,
        color: Color,
        text: &str,
    ) -> Result<(), String> {
        match (
            &self.font_info,
            &mut self.font_cache,
            &mut self.text_cache,
            &self.framebuffer,
        ) {
            (Some(font_info), Some(font_cache), Some(text_cache), Some(framebuffer_rc)) => {
                let framebuffer = framebuffer_rc.borrow_mut();

                match &framebuffer.attachments.forward_ldr {
                    Some(forward_ldr_rc) => {
                        let mut forward_ldr = forward_ldr_rc.borrow_mut();

                        let center_world_space = (Vec4::position(Default::default()) * *transform).to_vec3();

                        for plane in self.clipping_frustum.get_planes() {
                            if !plane.is_on_or_in_front_of(&center_world_space, 0.0) {
                                return Ok(());
                            }
                        }

                        let center_ndc_space = {
                            let shader_context = self.shader_context.borrow();

                            shader_context.to_ndc_space(center_world_space)
                        };

                        let center_viewport_space = Vec3 {
                            x: center_ndc_space.x * self.viewport.width as f32,
                            y: center_ndc_space.y * self.viewport.height as f32,
                            z: 0.0,
                        };

                        let (width, height) = cache_text(
                            font_cache,
                            text_cache,
                            font_info,
                            text,
                        );

                        let op = TextOperation {
                            text: &text.to_string(),
                            x: (center_viewport_space.x - width as f32 / 2.0) as u32,
                            y: (center_viewport_space.y - height as f32 / 2.0) as u32,
                            color,
                        };

                        Graphics::text(&mut forward_ldr, font_cache, Some(text_cache), font_info, &op)
                    },
                    None => Err("Called `SoftwareRenderer::render_text()` with no forward (LDR) attachment!".to_string()),
                }
            }
            _ => Err("Error! Failed!".to_string()),
        }
    }
}
