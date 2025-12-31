use bitflags::bitflags;

use cairo::{
    buffer::Buffer2D, color::Color,
    physics::simulation::physical_constants::EARTH_GRAVITY_ACCELERATION, vec::vec3::Vec3,
};

use crate::graph::Graph;

#[derive(Debug)]
pub struct Pendulum {
    pub suspension_point: Vec3,
    pub string_length: f32,
    pub bob_acceleration: Vec3,
    pub bob_velocity: Vec3,
    pub bob_position: Vec3,
    pub bob_position_history: Vec<Vec3>,
    pub bob_position_error_prev: f32,
    pub bob_position_error_accumulated: f32,
}

impl Default for Pendulum {
    fn default() -> Self {
        let suspension_height = 1.0;

        let string_length = 1.0;

        let bob_position = Vec3 {
            x: -string_length,
            y: suspension_height,
            ..Default::default()
        };

        Self {
            suspension_point: Vec3 {
                y: suspension_height,
                ..Default::default()
            },
            string_length,
            bob_acceleration: Default::default(),
            bob_velocity: Default::default(),
            bob_position,
            bob_position_history: vec![bob_position],
            bob_position_error_prev: 0.0,
            bob_position_error_accumulated: 0.0,
        }
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct PIDControllerFlags: u32 {
        const P = 1;
        const I = 1 << 1;
        const D = 1 << 2;
    }
}

impl Default for PIDControllerFlags {
    fn default() -> Self {
        Self::P | Self::I | Self::D
    }
}

#[derive(Debug)]
pub struct PIDController {
    pub k_p: f32,
    pub k_i: f32,
    pub k_d: f32,
    pub flags: PIDControllerFlags,
}

impl Default for PIDController {
    fn default() -> Self {
        Self {
            k_p: 6.0,
            k_i: 0.1,
            k_d: 3.0,
            flags: Default::default(),
        }
    }
}

impl PIDController {
    pub fn get_correction_force(
        &self,
        error: f32,
        normal: Vec3,
        derivative: f32,
        integral: f32,
    ) -> Vec3 {
        let mut force = Vec3::default();

        // Proportional controller term

        if self.flags.contains(PIDControllerFlags::P) {
            force += normal * error * self.k_p;
        }

        // Integral controller term

        if self.flags.contains(PIDControllerFlags::I) {
            force += normal * integral * self.k_i;
        }

        // Derivative controller term

        if self.flags.contains(PIDControllerFlags::D) {
            force += normal * derivative * self.k_d;
        }

        return force;
    }
}

#[derive(Debug)]
pub struct Simulation {
    pub pendulum: Pendulum,
    pub controller: PIDController,
    pub color: Color,
}

impl Simulation {
    pub fn new(flags: PIDControllerFlags, color: Color) -> Self {
        Self {
            pendulum: Default::default(),
            controller: PIDController {
                flags,
                ..Default::default()
            },
            color,
        }
    }

    pub fn tick(&mut self, delta_t: f32) {
        // Accumulate bob acceleration.
        self.pendulum.bob_acceleration = self.compute_acceleration(delta_t);

        // Integrate bob velocity.
        self.pendulum.bob_velocity += self.pendulum.bob_acceleration * delta_t;

        // Integrate bob position.
        self.pendulum.bob_position += self.pendulum.bob_velocity * delta_t;

        self.pendulum
            .bob_position_history
            .push(self.pendulum.bob_position);
    }

    pub fn render(&self, graph: &Graph, framebuffer: &mut Buffer2D) {
        let pendulum = &self.pendulum;

        // Draws the history (last 100 ticks) of the bob's position.

        const DRAWN_HISTORY_TICK_WINDOW: usize = 100;

        let drawn_history = pendulum
            .bob_position_history
            .iter()
            .rev()
            .take(DRAWN_HISTORY_TICK_WINDOW)
            .rev();

        let drawn_history_len = drawn_history.len();

        let mut prev: Option<Vec3> = None;

        for (i, next) in drawn_history.enumerate() {
            if let Some(prev) = prev {
                let alpha = i as f32 / drawn_history_len as f32;

                let color = self.color * alpha;

                graph.line(prev.x, prev.y, next.x, next.y, color, framebuffer);
            }

            prev.replace(*next);
        }

        // Draws the pendulum string.

        graph.line(
            pendulum.suspension_point.x,
            pendulum.suspension_point.y,
            pendulum.bob_position.x,
            pendulum.bob_position.y,
            self.color,
            framebuffer,
        );
    }

    fn error(&self) -> (f32, Vec3) {
        let r = self.pendulum.bob_position - self.pendulum.suspension_point;

        let current_length = r.mag();
        let desired_length = self.pendulum.string_length;

        let error = current_length - desired_length;

        let normal = -r.as_normal();

        return (error, normal);
    }

    fn compute_acceleration(&mut self, delta_t: f32) -> Vec3 {
        let mut external_forces = vec![EARTH_GRAVITY_ACCELERATION];

        // Adds a constraint force that preserves the length of the pendulum.

        let (error, normal) = self.error();

        self.pendulum.bob_position_error_accumulated += error;

        let integral = self.pendulum.bob_position_error_accumulated;

        let derivative = (error - self.pendulum.bob_position_error_prev) / delta_t;

        let corrective_force = self
            .controller
            .get_correction_force(error, normal, derivative, integral);

        self.pendulum.bob_position_error_prev = error;

        external_forces.push(corrective_force);

        // Sum up the external forces

        let net_force = external_forces
            .iter()
            .fold(Default::default(), |net_force, force| net_force + force);

        return net_force;
    }
}
