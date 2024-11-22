use std::f32::consts::PI;

use cairo::{
    buffer::Buffer2D,
    color,
    graphics::Graphics,
    matrix::Mat4,
    transform::Transform3D,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::{
    coordinates::{world_to_screen_space, PIXELS_PER_METER},
    renderable::Renderable,
    rigid_body_simulation_state::RigidBodySimulationState,
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
    #[allow(unused)]
    pub mass: f32,
    #[allow(unused)]
    pub moment_of_inertia: Mat4,
    pub transform: Transform3D,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
    pub kind: RigidBodyKind,
    // Derived state
    inverse_mass: f32,
    inverse_moment_of_inertia: Mat4,
    velocity: Vec3,
    angular_velocity: Vec3,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::circle(Default::default(), 1.0, 1.0)
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

impl RigidBody {
    pub fn circle(center: Vec3, radius: f32, mass: f32) -> Self {
        let (moment_of_inertia, inverse_moment_of_inertia) =
            get_moment_of_intertia_for_circle(radius);

        let mut transform = Transform3D::default();

        transform.set_translation(center);

        let mut result = Self {
            kind: RigidBodyKind::Circle(radius),
            mass,
            inverse_mass: 1.0 / mass,
            transform,
            linear_momentum: Default::default(),
            moment_of_inertia,
            inverse_moment_of_inertia,
            angular_momentum: Default::default(),
            velocity: Default::default(),
            angular_velocity: Default::default(),
        };

        result.recompute_derived_state();

        result
    }

    pub fn state(&self) -> RigidBodySimulationState {
        RigidBodySimulationState {
            inverse_mass: self.inverse_mass,
            inverse_moment_of_interia: self.inverse_moment_of_inertia,
            position: *self.transform.translation(),
            orientation: *self.transform.rotation(),
            linear_momentum: self.linear_momentum,
            angular_momentum: self.angular_momentum,
        }
    }

    pub fn apply_simulation_state(&mut self, state: &RigidBodySimulationState) {
        let (translation, mut orientation) = (state.position, state.orientation);

        self.transform.set_translation(translation);

        orientation.renormalize();

        self.transform.set_rotation(orientation);

        self.linear_momentum = state.linear_momentum;

        self.angular_momentum = state.angular_momentum;
    }

    fn recompute_derived_state(&mut self) {
        self.velocity = self.linear_momentum * self.inverse_mass;

        self.angular_velocity =
            (Vec4::new(self.angular_momentum, 0.0) * self.inverse_moment_of_inertia).to_vec3();
    }
}

impl Renderable for RigidBody {
    fn render(&self, buffer: &mut Buffer2D) {
        let position_screen_space = world_to_screen_space(self.transform.translation(), buffer);

        match self.kind {
            RigidBodyKind::Circle(radius) => {
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
                let global_right = local_right * *self.transform.rotation().mat();

                let end = *self.transform.translation() + (global_right * radius);
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
    }
}
