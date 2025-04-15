use crate::{physics::material::PhysicsMaterial, vec::vec3::Vec3};

pub fn resolve_plane_collision_approximate(
    plane_normal: Vec3,
    material: &PhysicsMaterial,
    end_position: &mut Vec3,
    end_velocity: &mut Vec3,
    penetration_depth: f32,
) {
    // Compute elasticity response (in the normal direction).

    let velocity_normal_to_plane = plane_normal * end_velocity.dot(plane_normal);

    let response_velocity_normal_to_plane = -velocity_normal_to_plane * material.restitution;

    // Compute friction response (in the tangent direction).

    let velocity_tangent_to_plane = *end_velocity - velocity_normal_to_plane;

    let loss = (material.dynamic_friction * velocity_normal_to_plane.mag())
        .min(velocity_tangent_to_plane.mag());

    let response_velocity_tangent_to_plane = if velocity_tangent_to_plane.is_zero() {
        velocity_tangent_to_plane
    } else {
        velocity_tangent_to_plane - velocity_tangent_to_plane.as_normal() * loss
    };

    let new_velocity = response_velocity_normal_to_plane + response_velocity_tangent_to_plane;

    let bias = if response_velocity_normal_to_plane.mag() < 0.05 {
        0.005
    } else {
        0.0
    };

    // Comptues a minimum displacement vector (accounting for restitution).

    let minimum_displacement =
        plane_normal * ((penetration_depth * (1.0 + material.restitution)) + bias);

    *end_velocity = new_velocity;

    *end_position += minimum_displacement;
}
