use super::SoftwareRenderer;

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_deferred_lighting_pass(&mut self) {
        if let Some(rc) = &self.framebuffer {
            let mut framebuffer = rc.borrow_mut();

            let attachments = &mut framebuffer.attachments;

            if let (Some(stencil_buffer_rc), Some(deferred_buffer_rc)) =
                (&attachments.stencil, attachments.deferred_hdr.as_mut())
            {
                let stencil_buffer = stencil_buffer_rc.borrow();

                let mut deferred_buffer = deferred_buffer_rc.borrow_mut();

                // Perform deferred lighting pass.

                let shader_context = self.shader_context.borrow();

                // Call the active fragment shader on every G-buffer sample that was
                // written to by the rasterizer.

                for (index, (stencil, sample)) in stencil_buffer
                    .0
                    .iter()
                    .zip(self.g_buffer.as_ref().unwrap().iter())
                    .enumerate()
                {
                    if *stencil != 0 {
                        let hdr_color = self.get_hdr_color_for_sample(
                            &shader_context,
                            &self.scene_resources,
                            sample,
                        );

                        deferred_buffer.set_at(index, hdr_color);
                    }
                }
            }
        }
    }
}
