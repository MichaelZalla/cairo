use crate::{
    color::blend::BlendMode, effect::Effect, effects::guassian_blur::GaussianBlurEffect,
    vec::vec3::Vec3,
};

use super::Pipeline;

impl<'a> Pipeline<'a> {
    pub(in crate::pipeline) fn do_deferred_lighting_pass(&mut self) {
        match self.framebuffer {
            Some(rc) => {
                let mut framebuffer = rc.borrow_mut();

                match framebuffer.attachments.forward_or_deferred_hdr.as_mut() {
                    Some(deferred_buffer_lock) => {
                        let mut deferred_buffer = deferred_buffer_lock.borrow_mut();

                        // Perform deferred lighting pass.

                        let shader_context = self.shader_context.borrow();

                        // Call the active fragment shader on every G-buffer sample that was
                        // written to by the rasterizer.

                        for (index, sample) in self.g_buffer.as_ref().unwrap().iter().enumerate() {
                            if sample.stencil == true {
                                let x = index as u32 % self.viewport.width;
                                let y = index as u32 / self.viewport.width;

                                let color = self.get_hdr_color_for_sample(&shader_context, &sample);

                                deferred_buffer.set(x, y, color);
                            }
                        }
                    }
                    None => (),
                }
            }
            None => (),
        }
    }

    pub(in crate::pipeline) fn do_bloom_pass(&mut self) {
        match self.framebuffer {
            Some(rc) => {
                let mut framebuffer = rc.borrow_mut();

                match framebuffer.attachments.forward_or_deferred_hdr.as_mut() {
                    Some(deferred_buffer_lock) => {
                        let mut deferred_buffer = deferred_buffer_lock.borrow_mut();

                        let mut bloom_frame = self.bloom_buffer.as_mut().unwrap();

                        for y in 0..deferred_buffer.height {
                            for x in 0..deferred_buffer.width {
                                let color_hdr = *deferred_buffer.get(x, y);

                                let perceived_brightness = if color_hdr.x >= 0.95
                                    || color_hdr.y >= 0.95
                                    || color_hdr.z >= 0.95
                                {
                                    1.0
                                } else {
                                    0.0
                                };

                                if perceived_brightness >= 1.0 {
                                    // Write this bright pixel to the initial bloom buffer.

                                    bloom_frame.set(x, y, color_hdr);
                                }
                            }
                        }

                        // Blur the bloom buffer.

                        let bloom_effect = GaussianBlurEffect::new(6);

                        bloom_effect.apply(&mut bloom_frame);

                        // Blit the bloom buffer to our composite framebuffer.

                        deferred_buffer.blit_blended_from(
                            0,
                            0,
                            &bloom_frame,
                            Some(BlendMode::Screen),
                            Some(Vec3::ones()),
                        );
                    }
                    None => (),
                }
            }
            None => panic!(),
        }
    }
}
