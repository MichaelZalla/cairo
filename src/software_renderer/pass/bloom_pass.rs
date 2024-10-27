use crate::{
    color::blend::BlendMode, effect::Effect, effects::gaussian_blur::GaussianBlurEffect,
    vec::vec3::Vec3,
};

use super::SoftwareRenderer;

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_bloom_pass(&mut self) {
        if self.bloom_effect.is_none() {
            self.bloom_effect.replace(GaussianBlurEffect::new(6));
        }

        match &self.framebuffer {
            Some(rc) => {
                let mut framebuffer = rc.borrow_mut();

                if let Some(deferred_buffer_rc) =
                    framebuffer.attachments.forward_or_deferred_hdr.as_mut()
                {
                    let mut deferred_buffer = deferred_buffer_rc.borrow_mut();

                    if let Some(bloom_buffer) = self.bloom_buffer.as_mut() {
                        for y in 0..deferred_buffer.height {
                            for x in 0..deferred_buffer.width {
                                let color_hdr = *deferred_buffer.get(x, y);

                                static THRESHOLD: f32 = 0.95;

                                if color_hdr.x >= THRESHOLD
                                    || color_hdr.y >= THRESHOLD
                                    || color_hdr.z >= THRESHOLD
                                {
                                    // Write this bright pixel to the initial bloom buffer.

                                    bloom_buffer.set(x, y, color_hdr);
                                }
                            }
                        }

                        // Blur the bloom buffer.

                        if let Some(effect) = self.bloom_effect.as_mut() {
                            effect.apply(bloom_buffer);
                        }
                    }

                    // Blit the bloom buffer to our composite framebuffer.

                    if let Some(bloom_buffer) = self.bloom_buffer.as_ref() {
                        deferred_buffer.copy_blended(
                            bloom_buffer.data.as_slice(),
                            Some(BlendMode::Screen),
                            Some(Vec3::ones()),
                        );
                    }
                }
            }
            None => panic!(),
        }
    }
}
