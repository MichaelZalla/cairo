use std::f32::EPSILON;

use crate::vec::{
    vec3::{self, Vec3},
    vec4::Vec4,
};

use super::SoftwareRenderer;

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_weighted_blended_pass(&mut self) {
        match &self.framebuffer {
            Some(framebuffer_rc) => {
                let framebuffer = framebuffer_rc.borrow_mut();

                if let Some(deferred_buffer_rc) = framebuffer.attachments.deferred_hdr.as_ref() {
                    let mut deferred_buffer = deferred_buffer_rc.borrow_mut();

                    for ((hdr_color, accumulation), revealage) in deferred_buffer
                        .data
                        .iter_mut()
                        .zip(self.alpha_accumulation_buffer.data.iter_mut())
                        .zip(&self.alpha_revealage_buffer.data)
                    {
                        if *revealage > 1.0 - EPSILON {
                            continue;
                        }

                        *hdr_color = weighted_blended(*hdr_color, *accumulation, *revealage);
                    }
                }
            }
            None => panic!(),
        }
    }
}

fn weighted_blended(dest: Vec3, mut accumulation: Vec4, revealage: f32) -> Vec3 {
    // Check for floating-point overflow in any color channel.

    if accumulation.abs().max().is_infinite() {
        let alpha = accumulation.w;

        // Here, we simply replace the bad color with Vec3(alpha).
        accumulation = Vec4::new(vec3::ONES * alpha, alpha);
    }

    // Normalizes accumulated color by the total accumulated alpha (avoid
    // divide-by-zero).

    let normalized = accumulation.to_vec3() / accumulation.w.max(EPSILON);

    // Source: GL_SRC_ALPHA, dest: GL_ONE_MINUS_SRC_ALPHA

    let src_alpha = 1.0 - revealage;

    let src = Vec4::new(normalized, src_alpha);

    dest * (1.0 - src_alpha) + src.to_vec3() * src_alpha
}