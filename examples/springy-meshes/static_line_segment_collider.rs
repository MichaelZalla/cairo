use std::fmt;

use cairo::{
    geometry::primitives::plane::Plane,
    physics::material::PhysicsMaterial,
    vec::vec3::{self, Vec3},
};

pub(crate) trait Collider {
    fn test(&self, position: &Vec3, new_position: &Vec3) -> Option<(f32, f32)>;

    fn resolve_approximate(
        &self,
        new_position: &mut Vec3,
        new_velocity: &mut Vec3,
        new_distance: f32,
    );
}

#[derive(Default)]
pub(crate) struct StaticLineSegmentCollider {
    pub start: Vec3,
    pub end: Vec3,
    pub plane: Plane,
    tangent: Vec3,
    length: f32,
    material: PhysicsMaterial,
}

impl StaticLineSegmentCollider {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        let point = start + (end - start) / 2.0;

        let delta = end - start;
        let length = delta.mag();
        let tangent = delta.as_normal();
        let normal = vec3::FORWARD.cross(tangent).as_normal();

        let plane = Plane::new(point, normal);

        Self {
            start,
            end,
            plane,
            tangent,
            length,
            material: PhysicsMaterial {
                restitution: 0.8,
                dynamic_friction: 0.2,
            },
        }
    }
}

impl Collider for StaticLineSegmentCollider {
    fn test(&self, position: &Vec3, new_position: &Vec3) -> Option<(f32, f32)> {
        let projection = (*new_position - self.start).dot(self.tangent);

        if projection < 0.0 || projection > self.length {
            return None;
        }

        let distance = self.plane.get_signed_distance(position);
        let new_distance = self.plane.get_signed_distance(new_position);

        if (distance * new_distance) < 0.0 {
            // Calculates the fraction of the timestep at which the collision
            // occurred; note that this calculation assumes constant
            // acceleration of the collider's 2 points and the (separate) point
            // being tested.

            let delta = distance - new_distance;

            let f = distance / delta;

            Some((f, new_distance))
        } else {
            None
        }
    }

    fn resolve_approximate(
        &self,
        new_position: &mut Vec3,
        new_velocity: &mut Vec3,
        new_distance: f32,
    ) {
        // Compute elasticity response (in the normal direction).

        let velocity_normal_to_plane = self.plane.normal * new_velocity.dot(self.plane.normal);

        let response_velocity_normal_to_plane =
            -velocity_normal_to_plane * self.material.restitution;

        // Compute friction response (in the tangent direction).

        let velocity_tangent_to_plane = *new_velocity - velocity_normal_to_plane;

        let loss = (self.material.dynamic_friction * velocity_normal_to_plane.mag())
            .min(velocity_tangent_to_plane.mag());

        let response_velocity_tangent_to_plane =
            velocity_tangent_to_plane - velocity_tangent_to_plane.as_normal() * loss;

        *new_velocity = response_velocity_normal_to_plane + response_velocity_tangent_to_plane;

        *new_position -= self.plane.normal * (1.0 + self.material.restitution) * new_distance;
    }
}

impl fmt::Display for StaticLineSegmentCollider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StaticLineSegmentCollider (start={}, end={})",
            self.start, self.end
        )
    }
}
