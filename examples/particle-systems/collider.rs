use std::fmt;

use cairo::{
    physics::collider::plane::Plane,
    vec::vec3::{self, Vec3},
};

static DEFAULT_COEFFICIENT_OF_RESTITUTION: f32 = 0.75;
static DEFAULT_COEFFICIENT_OF_FRICTION: f32 = 0.2;

#[derive(Default)]
pub(crate) struct LineSegmentCollider {
    pub start: Vec3,
    pub end: Vec3,
    pub plane: Plane,
    tangent: Vec3,
    length: f32,
    restitution: f32,
    friction: f32,
}

impl LineSegmentCollider {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        let point = start + (end - start) / 2.0;

        let delta = end - start;
        let length = delta.mag();
        let tangent = delta.as_normal();
        let normal = vec3::FORWARD.cross(tangent).as_normal();

        let plane = Plane { point, normal };

        Self {
            start,
            end,
            plane,
            tangent,
            length,
            restitution: DEFAULT_COEFFICIENT_OF_RESTITUTION,
            friction: DEFAULT_COEFFICIENT_OF_FRICTION,
        }
    }

    pub fn get_post_collision_distance(&self, position: &Vec3, new_position: &Vec3) -> Option<f32> {
        let projection = (*new_position - self.start).dot(self.tangent);

        if projection < 0.0 || projection > self.length {
            return None;
        }

        let distance = self.plane.get_signed_distance_to_plane(position);
        let new_distance = self.plane.get_signed_distance_to_plane(new_position);

        if (distance * new_distance) < 0.0 {
            Some(new_distance)
        } else {
            None
        }
    }

    pub fn resolve_approximate(
        &self,
        new_position: &mut Vec3,
        new_velocity: &mut Vec3,
        new_distance: f32,
    ) {
        // Compute elasticity response (in the normal direction).

        let velocity_normal_to_plane = self.plane.normal * new_velocity.dot(self.plane.normal);

        let response_velocity_normal_to_plane = -velocity_normal_to_plane * self.restitution;

        // Compute friction response (in the tangent direction).

        let velocity_tangent_to_plane = *new_velocity - velocity_normal_to_plane;

        let loss =
            (self.friction * velocity_normal_to_plane.mag()).min(velocity_tangent_to_plane.mag());

        let response_velocity_tangent_to_plane =
            velocity_tangent_to_plane - velocity_tangent_to_plane.as_normal() * loss;

        *new_velocity = response_velocity_normal_to_plane + response_velocity_tangent_to_plane;

        *new_position = *new_position - self.plane.normal * (1.0 + self.restitution) * new_distance;
    }
}

impl fmt::Display for LineSegmentCollider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LineSegmentCollider (start={}, end={})",
            self.start, self.end
        )
    }
}
