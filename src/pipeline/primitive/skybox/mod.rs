use crate::{
    pipeline::{zbuffer, Pipeline},
    scene::camera::Camera,
    shader::{
        alpha::AlphaShader, fragment::FragmentShader, geometry::GeometryShader,
        vertex::VertexShader,
    },
    texture::cubemap::CubeMap,
};

impl<'a, F, V, A, G> Pipeline<'a, F, V, A, G>
where
    F: FragmentShader<'a>,
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
    pub fn render_skybox(&mut self, skybox: &CubeMap, camera: &Camera) {
        for (index, z_non_linear) in self.z_buffer.values.iter().enumerate() {
            // If this pixel was not shaded by our fragment shader

            if *z_non_linear == zbuffer::MAX_DEPTH {
                // Note: z_buffer_index = (y * self.graphics.buffer.width + x)

                let screen_x: u32 = (index as f32 % self.forward_framebuffer.width as f32) as u32;
                let screen_y: u32 = (index as f32 / self.forward_framebuffer.width as f32) as u32;

                let pixel_coordinate_world_space = camera.get_pixel_world_space_position(
                    screen_x,
                    screen_y,
                    self.forward_framebuffer.width,
                    self.forward_framebuffer.height,
                );

                let normal = pixel_coordinate_world_space.as_normal();

                // Sample the cubemap using our world-space direction-offset.

                let skybox_color = skybox.sample(&normal);

                self.forward_framebuffer
                    .set_pixel(screen_x, screen_y, skybox_color);
            }
        }
    }
}
