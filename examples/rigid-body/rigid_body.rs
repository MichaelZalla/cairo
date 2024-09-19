use std::f32::consts::PI;

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
    transform::Transform,
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
    pub mass: f32,
    pub moment_of_inertia: f32,
    pub transform: Transform,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
    pub kind: RigidBodyKind,
    // Derived state
    one_over_mass: f32,
    one_over_moment_of_inertia: f32,
    velocity: Vec3,
    angular_velocity: Vec3,
    spin: Quaternion,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::circle(Default::default(), 1.0, 1.0)
    }
}

impl RigidBody {
    pub fn circle(center: Vec3, radius: f32, mass: f32) -> Self {
        let moment_of_inertia = (PI * radius.powi(4)) / 4.0;
        let one_over_moment_of_inertia = 1.0 / moment_of_inertia;

        let mut result = Self {
            kind: RigidBodyKind::Circle(radius),
            mass,
            transform: Transform::new(center),
            linear_momentum: Default::default(),
            moment_of_inertia,
            one_over_moment_of_inertia,
            angular_momentum: Default::default(),
            // To be initialized...
            one_over_mass: 0.0,
            velocity: Default::default(),
            angular_velocity: Default::default(),
            spin: Default::default(),
        };

        result.recompute_derived_state();

        result
    }

    fn recompute_derived_state(&mut self) {
        self.one_over_mass = 1.0 / self.mass;
        self.velocity = self.linear_momentum * self.one_over_mass;

        self.one_over_moment_of_inertia = 1.0 / self.one_over_moment_of_inertia;
        self.angular_velocity = self.angular_momentum * self.one_over_moment_of_inertia;

        let angular_velocity_q = Quaternion::new(self.angular_velocity, 0.0);

        self.spin = angular_velocity_q * 0.5 * (*self.transform.orientation());
    }
}

impl Renderable for RigidBody {
    fn render(&self, buffer: &mut Buffer2D) {
        let position_screen_space = world_to_screen_space(self.transform.translation(), &buffer);

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
                let global_right = (local_right * *self.transform.orientation().mat()).to_vec3();

                let end = *self.transform.translation() + (global_right * radius);
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
