use super::SoftwareRenderer;

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_tone_mapping_pass(&mut self) {
        match &self.framebuffer {
            Some(framebuffer_rc) => {
                let framebuffer = framebuffer_rc.borrow_mut();

                if let (Some(deferred_buffer_rc), Some(color_buffer_rc)) = (
                    framebuffer.attachments.forward_or_deferred_hdr.as_ref(),
                    framebuffer.attachments.color.as_ref(),
                ) {
                    let deferred_buffer = deferred_buffer_rc.borrow();

                    let mut color_buffer = color_buffer_rc.borrow_mut();

                    for (hdr_color, entry) in deferred_buffer.iter().zip(color_buffer.iter_mut()) {
                        let lit_geometry_fragment_color_tone =
                            self.get_tone_mapped_color_from_hdr(*hdr_color);

                        *entry = lit_geometry_fragment_color_tone.to_u32();
                    }
                }
            }
            None => panic!(),
        }
    }
}
