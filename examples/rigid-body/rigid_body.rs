use std::f32::consts::PI;

use cairo::{
    buffer::Buffer2D,
    color,
    graphics::Graphics,
    matrix::Mat4,
    physics::simulation::rigid_body::RigidBody,
    transform::Transform3D,
    vec::vec3::{self, Vec3},
};

use crate::{
    coordinates::{world_to_screen_space, PIXELS_PER_METER},
    renderable::Renderable,
};

pub struct CircleRigidBody {
    pub radius: f32,
    pub rigid_body: RigidBody,
}

impl Default for CircleRigidBody {
    fn default() -> Self {
        Self::new(Default::default(), 1.0, 1.0)
    }
}

impl CircleRigidBody {
    pub fn new(center: Vec3, radius: f32, mass: f32) -> Self {
        let (moment_of_inertia, inverse_moment_of_inertia) =
            get_moment_of_intertia_for_circle(radius);

        let mut transform = Transform3D::default();

        transform.set_translation(center);

        let rigid_body = RigidBody::new(
            mass,
            transform,
            moment_of_inertia,
            inverse_moment_of_inertia,
        );

        Self { radius, rigid_body }
    }
}

fn get_moment_of_intertia_for_circle(radius: f32) -> (Mat4, Mat4) {
    let scale = (PI * radius.powi(4)) / 4.0;

    let moment_of_inertia = { Mat4::scale([scale, scale, scale, 1.0]) };

    let inverse_moment_of_inertia = {
        let inverse_scale = 1.0 / scale;

        Mat4::scale([inverse_scale, inverse_scale, inverse_scale, 1.0])
    };

    (moment_of_inertia, inverse_moment_of_inertia)
}

impl Renderable for CircleRigidBody {
    fn render(&self, buffer: &mut Buffer2D) {
        let transform = &self.rigid_body.transform;

        let position_screen_space = world_to_screen_space(transform.translation(), buffer);

        // Draw the circle's outline.

        Graphics::circle(
            buffer,
            position_screen_space.x as i32,
            position_screen_space.y as i32,
            (self.radius * PIXELS_PER_METER) as u32,
            None,
            Some(color::YELLOW.to_u32()),
        );

        // Draw a line to indicate the body's orientation.

        let local_right = vec3::RIGHT;
        let global_right = local_right * *transform.rotation().mat();

        let end = *transform.translation() + (global_right * self.radius);
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
}
