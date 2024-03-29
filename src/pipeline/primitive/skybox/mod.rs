use crate::{
    pipeline::{zbuffer, Pipeline},
    scene::camera::Camera,
    texture::cubemap::CubeMap,
};

impl<'a> Pipeline<'a> {
    pub fn render_skybox(&mut self, skybox: &CubeMap, camera: &Camera) {
        match self.framebuffer {
            Some(rc) => {
                let framebuffer = rc.borrow_mut();

                match (
                    framebuffer.attachments.depth.as_ref(),
                    framebuffer.attachments.forward_ldr.as_ref(),
                ) {
                    (Some(depth_buffer_lock), Some(forward_buffer_lock)) => {
                        let mut depth_buffer = depth_buffer_lock.borrow_mut();
                        let mut forward_buffer = forward_buffer_lock.borrow_mut();

                        for (index, z_non_linear) in depth_buffer.iter().enumerate() {
                            // If this pixel was not shaded by our fragment shader

                            if *z_non_linear == zbuffer::MAX_DEPTH {
                                // Note: z_buffer_index = (y * self.graphics.buffer.width + x)

                                let screen_x: u32 =
                                    (index as f32 % self.viewport.width as f32) as u32;
                                let screen_y: u32 =
                                    (index as f32 / self.viewport.width as f32) as u32;

                                let pixel_coordinate_world_space = camera
                                    .get_near_plane_pixel_world_space_position(
                                        screen_x,
                                        screen_y,
                                        self.viewport.width,
                                        self.viewport.height,
                                    );

                                let normal = pixel_coordinate_world_space.as_normal();

                                // Sample the cubemap using our world-space direction-offset.

                                let skybox_color = skybox.sample(&normal);

                                forward_buffer.set(screen_x, screen_y, skybox_color.to_u32());
                            }
                        }
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }
}
