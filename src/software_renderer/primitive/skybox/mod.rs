use crate::{
    matrix::Mat4,
    scene::camera::Camera,
    software_renderer::{zbuffer, SoftwareRenderer},
    texture::cubemap::CubeMap,
    vec::vec3::Vec3,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_skybox(
        &mut self,
        skybox: &CubeMap,
        camera: &Camera,
        skybox_rotation: Option<Mat4>,
    ) {
        if let Some(framebuffer_rc) = &self.framebuffer {
            let framebuffer = framebuffer_rc.borrow_mut();

            if let (Some(depth_buffer_rc), Some(forward_buffer_rc)) = (
                framebuffer.attachments.depth.as_ref(),
                framebuffer.attachments.forward_ldr.as_ref(),
            ) {
                let mut depth_buffer = depth_buffer_rc.borrow_mut();
                let mut forward_buffer = forward_buffer_rc.borrow_mut();

                for (index, z_non_linear) in depth_buffer.iter().enumerate() {
                    // If this pixel was not shaded by our fragment shader

                    if *z_non_linear == zbuffer::MAX_DEPTH {
                        // Note: z_buffer_index = (y * self.graphics.buffer.width + x)

                        let screen_x: u32 = (index as f32 % self.viewport.width as f32) as u32;
                        let screen_y: u32 = (index as f32 / self.viewport.width as f32) as u32;

                        let pixel_coordinate_world_space = camera
                            .get_near_plane_pixel_world_space_position(
                                screen_x,
                                screen_y,
                                self.viewport.width,
                                self.viewport.height,
                            );

                        let mut normal = pixel_coordinate_world_space.as_normal();

                        if let Some(transform) = skybox_rotation {
                            normal *= transform;
                        }

                        // Sample the cubemap using our world-space direction-offset.

                        let skybox_color = if self.shader_options.bilinear_active {
                            skybox.sample_bilinear(&normal, None)
                        } else {
                            skybox.sample_nearest(&normal, None)
                        };

                        forward_buffer.set(screen_x, screen_y, skybox_color.to_u32());
                    }
                }
            }
        }
    }

    pub(in crate::software_renderer) fn _render_skybox_hdr(
        &mut self,
        skybox_hdr: &CubeMap<Vec3>,
        camera: &Camera,
        skybox_rotation: Option<Mat4>,
    ) {
        if let Some(framebuffer_rc) = &self.framebuffer {
            let framebuffer = framebuffer_rc.borrow_mut();

            if let (Some(stencil_buffer_rc), Some(forward_buffer_rc)) = (
                framebuffer.attachments.stencil.as_ref(),
                framebuffer.attachments.forward_ldr.as_ref(),
            ) {
                let stencil_buffer = stencil_buffer_rc.borrow();

                let mut forward_buffer = forward_buffer_rc.borrow_mut();

                for (index, written) in stencil_buffer.0.iter().enumerate() {
                    // If this pixel was not shaded by our fragment shader

                    if *written == 0 {
                        let x: u32 = (index as f32 % self.viewport.width as f32) as u32;
                        let y: u32 = (index as f32 / self.viewport.width as f32) as u32;

                        let pixel_coordinate_world_space = camera
                            .get_near_plane_pixel_world_space_position(
                                x,
                                y,
                                self.viewport.width,
                                self.viewport.height,
                            );

                        let mut normal = pixel_coordinate_world_space.as_normal();

                        if let Some(transform) = skybox_rotation {
                            normal *= transform;
                        }

                        // Sample the cubemap using our world-space direction-offset.

                        let skybox_hdr_color = skybox_hdr.sample_nearest(&normal, None);

                        let skybox_color = self.get_tone_mapped_color_from_hdr(skybox_hdr_color);

                        forward_buffer.set(x, y, skybox_color.to_u32());
                    }
                }
            }
        }
    }
}
