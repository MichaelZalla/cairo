use super::SoftwareRenderer;

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_deferred_lighting_pass(&mut self) {
        if let Some(rc) = &self.framebuffer {
            let mut framebuffer = rc.borrow_mut();

            if let Some(deferred_buffer_lock) =
                framebuffer.attachments.forward_or_deferred_hdr.as_mut()
            {
                let mut deferred_buffer = deferred_buffer_lock.borrow_mut();

                // Perform deferred lighting pass.

                let shader_context = self.shader_context.borrow();

                // Call the active fragment shader on every G-buffer sample that was
                // written to by the rasterizer.

                for (index, sample) in self.g_buffer.as_ref().unwrap().iter().enumerate() {
                    if sample.stencil {
                        let x = index as u32 % self.viewport.width;
                        let y = index as u32 / self.viewport.width;

                        let color = self.get_hdr_color_for_sample(
                            &shader_context,
                            &self.scene_resources,
                            sample,
                        );

                        deferred_buffer.set(x, y, color);
                    }
                }
            }
        }
    }
}
