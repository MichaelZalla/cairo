use std::ops;

use crate::{
    geometry::primitives::aabb::{Bounded, AABB},
    matrix::Mat4,
    physics::simulation::{
        contact::{StaticContactKind, StaticContactList},
        force::{DynForce, Force},
    },
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::RigidBodyKind;

pub type RigidBodyForce = Force<RigidBodySimulationState>;
pub type DynRigidBodyForce = DynForce<RigidBodySimulationState>;

#[derive(Default, Debug, Copy, Clone)]
pub struct RigidBodySimulationState {
    pub kind: RigidBodyKind,
    pub inverse_mass: f32,
    pub inverse_moment_of_inertia: Mat4,
    pub position: Vec3,
    pub orientation: Quaternion,
    pub linear_momentum: Vec3,
    pub angular_momentum: Vec3,
    pub static_contacts: StaticContactList<6>,
}

impl ops::AddAssign for RigidBodySimulationState {
    fn add_assign(&mut self, rhs: Self) {
        self.position += rhs.position;
        self.orientation += rhs.orientation;
        self.linear_momentum += rhs.linear_momentum;
        self.angular_momentum += rhs.angular_momentum;
    }
}

impl ops::Add for RigidBodySimulationState {
    type Output = RigidBodySimulationState;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl ops::MulAssign for RigidBodySimulationState {
    fn mul_assign(&mut self, rhs: Self) {
        self.position *= rhs.position;
        self.orientation *= rhs.orientation;
        self.linear_momentum *= rhs.linear_momentum;
        self.angular_momentum *= rhs.angular_momentum;
    }
}

impl ops::Mul for RigidBodySimulationState {
    type Output = RigidBodySimulationState;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl ops::Mul<f32> for RigidBodySimulationState {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        let mut result = self;

        result.position *= scalar;
        result.orientation *= scalar;
        result.linear_momentum *= scalar;
        result.angular_momentum *= scalar;

        result
    }
}

impl Bounded for RigidBodySimulationState {
    fn aabb(&self) -> AABB {
        match self.kind {
            RigidBodyKind::Circle(_) => panic!("Cannot produce an AABB from a 2D circle!"),
            RigidBodyKind::Sphere(radius) => {
                let offset = vec3::ONES * radius;

                AABB::from((self.position - offset, self.position + offset))
            }
        }
    }
}

impl RigidBodySimulationState {
    pub fn velocity(&self) -> Vec3 {
        self.linear_momentum * self.inverse_mass
    }

    pub fn inverse_moment_of_inertia_world_space(&self) -> Mat4 {
        let r = *self.orientation.mat();

        r * self.inverse_moment_of_inertia * r.transposed()
    }

    pub fn angular_velocity(&self) -> Vec3 {
        let angular_momentum = Vec4::vector(self.angular_momentum);

        let inverse_moment_of_inertia_world_space = self.inverse_moment_of_inertia_world_space();

        (angular_momentum * inverse_moment_of_inertia_world_space).to_vec3()
    }

    pub fn angular_velocity_quaternion(&self) -> Quaternion {
        let angular_velocity = self.angular_velocity();

        let spin = Quaternion::from_raw(0.0, angular_velocity);

        // First-order integration (assumes that velocity is constant over the timestep).
        //
        // See: https://stackoverflow.com/a/46924782/1623811
        // See: https://www.ashwinnarayan.com/post/how-to-integrate-quaternions/

        self.orientation * 0.5 * spin
    }

    pub fn accumulate_accelerations(
        &self,
        forces: &[Box<DynRigidBodyForce>],
        derivative: &mut Self,
        h: f32,
        current_time: f32,
    ) {
        let position = self.position;

        let mut total_acceleration = Vec3::default();
        let mut total_torque = Vec3::default();

        for force in forces {
            let (newtons, contact_point, is_gravity) = force(self, 0, current_time);

            // Accumulate linear momentum.

            let acceleration = if is_gravity {
                newtons
            } else {
                newtons * self.inverse_mass
            };

            total_acceleration += acceleration;

            // Accumulate angular momentum.

            if let Some(point) = contact_point {
                let r = point - position;

                total_torque += r.cross(acceleration);
            }
        }

        let mut remaining_total_acceleration = total_acceleration;

        for contact in &self.static_contacts {
            let external_force_magnitude_along_normal =
                contact.normal.dot(remaining_total_acceleration);

            if external_force_magnitude_along_normal < -0.001 {
                if let StaticContactKind::Resting = &contact.kind {
                    let external_force_magnitude_along_tangent =
                        contact.tangent.dot(remaining_total_acceleration);

                    let external_force_magnitude_along_tangent_required_to_slide =
                        (-external_force_magnitude_along_normal * contact.material.static_friction)
                            .min(0.001);

                    if external_force_magnitude_along_tangent
                        < external_force_magnitude_along_tangent_required_to_slide
                    {
                        // Applies static friction force, halting movement.

                        let body_linear_velocity = self.velocity();
                        let body_angular_velocity = self.angular_velocity();

                        let scale = 1.0 / h.max(0.00001);

                        let acceleration_needed_to_zero_linear_velocity =
                            -body_linear_velocity * scale;

                        remaining_total_acceleration = acceleration_needed_to_zero_linear_velocity;

                        let torque_needed_to_zero_angular_velocity =
                            -body_angular_velocity * scale * 0.999;

                        total_torque = torque_needed_to_zero_angular_velocity;
                    }
                }
            }

            let external_force_along_normal =
                contact.normal * external_force_magnitude_along_normal;

            remaining_total_acceleration -= external_force_along_normal;
        }

        derivative.linear_momentum += remaining_total_acceleration;
        derivative.angular_momentum += total_torque;
    }
}
