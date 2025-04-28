use cairo::{
    buffer::Buffer2D,
    color,
    graphics::Graphics,
    physics::simulation::rigid_body::{RigidBody, RigidBodyKind},
    vec::vec3,
};

use crate::{
    coordinates::{world_to_screen_space, PIXELS_PER_METER},
    renderable::Renderable,
};

impl Renderable for RigidBody {
    fn render(&self, buffer: &mut Buffer2D) {
        match self.kind {
            RigidBodyKind::Circle(radius) => {
                let transform = &self.transform;

                let position_screen_space = world_to_screen_space(transform.translation(), buffer);

                // Draw the circle's outline.

                Graphics::circle(
                    buffer,
                    position_screen_space.x as i32,
                    position_screen_space.y as i32,
                    (radius * PIXELS_PER_METER) as u32,
                    None,
                    Some(color::YELLOW.to_u32()),
                );

                // Draw a line to indicate the body's orientation.

                let local_right = vec3::RIGHT;
                let global_right = local_right * *transform.rotation().mat();

                let end = *transform.translation() + (global_right * radius);
                let end_screen_space = world_to_screen_space(&end, buffer);

                Graphics::line(
                    buffer,
                    position_screen_space.x as i32,
                    position_screen_space.y as i32,
                    end_screen_space.x as i32,
                    end_screen_space.y as i32,
                    color::ORANGE.to_u32(),
                );
            }
            _ => panic!(),
        }
    }
}
