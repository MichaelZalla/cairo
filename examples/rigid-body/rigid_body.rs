use cairo::{
    buffer::Buffer2D,
    color::{self},
    graphics::Graphics,
    vec::{vec3::Vec3, vec4},
};

use crate::{
    coordinates::{world_to_screen_space, PIXELS_PER_METER},
    quaternion::Quaternion,
    renderable::Renderable,
};

#[derive(Debug, Copy, Clone)]
pub enum RigidBodyKind {
    Circle(f32),
}

impl Default for RigidBodyKind {
    fn default() -> Self {
        Self::Circle(1.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RigidBody {
    pub position: Vec3,
    pub orientation: Quaternion,
    pub kind: RigidBodyKind,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            orientation: Quaternion::new_2d(0.0),
            position: Default::default(),
            kind: RigidBodyKind::Circle(1.0),
        }
    }
}

impl RigidBody {
    pub fn circle(center: Vec3, radius: f32) -> Self {
        Self {
            kind: RigidBodyKind::Circle(radius),
            position: center,
            ..Default::default()
        }
    }
}

impl Renderable for RigidBody {
    fn render(&self, buffer: &mut Buffer2D) {
        let position_screen_space = world_to_screen_space(&self.position, &buffer);

        match self.kind {
            RigidBodyKind::Circle(radius) => {
                // Draw the circle's outline.

                Graphics::circle(
                    buffer,
                    position_screen_space.x as u32,
                    position_screen_space.y as u32,
                    (radius * PIXELS_PER_METER) as u32,
                    None,
                    Some(&color::YELLOW),
                );

                // Draw a line to indicate the body's orientation.

                let local_right = vec4::RIGHT;
                let global_right = (local_right * *self.orientation.mat()).to_vec3();

                let end = self.position + (global_right * radius);
                let end_screen_space = world_to_screen_space(&end, &buffer);

                Graphics::line(
                    buffer,
                    position_screen_space.x as i32,
                    position_screen_space.y as i32,
                    end_screen_space.x as i32,
                    end_screen_space.y as i32,
                    &color::ORANGE,
                );
            }
        }
    }
}
