use cairo::{
    buffer::Buffer2D, color, graphics::Graphics, physics::simulation::particle::Particle,
    vec::vec3::Vec3,
};

use crate::{coordinates::world_to_screen_space, renderable::Renderable};

impl Renderable for Particle {
    fn render(&self, buffer: &mut Buffer2D, buffer_center: &Vec3) {
        static POINT_SIZE: u32 = 4;
        static POINT_SIZE_OVER_2: u32 = POINT_SIZE / 2;

        let world_space_position = self.position;

        let screen_space_position = world_to_screen_space(&world_space_position, buffer_center);

        let center_x = screen_space_position.x as i32;
        let center_y = screen_space_position.y as i32;

        if let Some((x, y, width, height)) = Graphics::clip_rectangle(
            buffer,
            center_x - POINT_SIZE_OVER_2 as i32,
            center_y - POINT_SIZE_OVER_2 as i32,
            POINT_SIZE,
            POINT_SIZE,
        ) {
            Graphics::rectangle(
                buffer,
                x,
                y,
                width,
                height,
                Some(color::YELLOW.to_u32()),
                None,
            )
        }
    }
}
