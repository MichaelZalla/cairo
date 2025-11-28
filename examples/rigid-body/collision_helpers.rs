use cairo::{
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::{
                NormalImpulseData, get_rigid_body_plane_friction_impulse,
                get_rigid_body_plane_normal_impulse,
            },
            rigid_body::{RigidBodyKind, rigid_body_simulation_state::RigidBodySimulationState},
        },
    },
    vec::vec3::Vec3,
};

use crate::{static_collider::CollisionSurface, static_contact_helpers::RESTING_SPEED_THRESHOLD};

/// Removes linear momentum accumulated after the collision point.
pub fn remove_post_collision_momentum(
    h: f32,
    collision_t: f32,
    body_derivative: &RigidBodySimulationState,
    body_state: &mut RigidBodySimulationState,
) {
    let time_before_collision = h * collision_t;
    let time_after_collision = h - time_before_collision;

    let linear_acceleration = body_derivative.linear_momentum;
    let mass = 1.0 / body_state.inverse_mass;

    let accumulated_linear_velocity = linear_acceleration * time_after_collision;

    body_state.linear_momentum -= accumulated_linear_velocity * mass;
}

/// Calculates collision response data if a collision is detected.
///
/// Returns `Some((normal_impulse_data, collision_t))` if a collision occurred,
/// where `collision_t` is the normalized time of collision within the timestep.
/// Returns `None` if no collision occurred or if the contact point is not
/// moving into the plane.
pub fn calculate_collision_response(
    h: f32,
    body_derivative: &RigidBodySimulationState,
    new_body_state: &mut RigidBodySimulationState,
    surface: &CollisionSurface,
    material: &PhysicsMaterial,
) -> Option<(NormalImpulseData, f32)> {
    let end_position = new_body_state.position;
    let end_linear_velocity = new_body_state.velocity();

    if surface.collision_t > 1.0 {
        // Future collision, ignore for this tick
        return None;
    }

    // For sphere contacts, recalculate contact point from end position
    // to avoid spurious offset from capsule intersection test
    let contact_point_corrected = match new_body_state.kind {
        RigidBodyKind::Sphere(_radius) => {
            // Contact point is directly below sphere center along normal
            end_position - surface.normal * surface.minimum_distance
        }
        _ => surface.contact_point,
    };

    // For torque calculation, moment arm points FROM contact TO center
    let r_torque = end_position - contact_point_corrected;

    // For contact velocity, offset points FROM center TO contact
    let r_velocity = contact_point_corrected - end_position;

    // Compute linear velocity of the contact point in world space
    let contact_point_velocity =
        end_linear_velocity + new_body_state.angular_velocity().cross(r_velocity);

    // Check if contact point is moving into the surface
    let incoming_speed = contact_point_velocity.dot(surface.normal);

    if incoming_speed < -RESTING_SPEED_THRESHOLD {
        // Remove momentum accumulated after collision
        remove_post_collision_momentum(h, surface.collision_t, body_derivative, new_body_state);

        // Calculate normal impulse
        Some((
            get_rigid_body_plane_normal_impulse(
                new_body_state,
                surface.normal,
                surface.contact_point,
                contact_point_velocity,
                r_velocity,
                r_torque,
                material,
            ),
            surface.collision_t,
        ))
    } else {
        None
    }
}

/// Applies collision impulses (normal and friction) to the body state.
///
/// This function:
/// 1. Applies the normal impulse to linear and angular momentum
/// 2. Updates the body's position based on pre/post-collision velocities
/// 3. Calculates and applies friction impulse (if any)
#[allow(clippy::too_many_arguments)]
pub fn apply_collision_impulses(
    h: f32,
    body_derivative: &RigidBodySimulationState,
    body_state: &mut RigidBodySimulationState,
    start_position: Vec3,
    start_linear_velocity: Vec3,
    normal_impulse_data: &NormalImpulseData,
    collision_t: f32,
    material: &PhysicsMaterial,
) {
    let normal = normal_impulse_data.normal;
    let normal_impulse_magnitude = normal_impulse_data.magnitude;
    let r_torque = normal_impulse_data.r;

    let normal_impulse = normal * normal_impulse_magnitude;

    // Applies normal impulse to linear momentum.
    body_state.linear_momentum += normal_impulse;

    // Applies normal impulse to angular momentum.
    // Torque = r × impulse (must apply to the full impulse vector)
    body_state.angular_momentum += r_torque.cross(normal_impulse);

    // Determines the body's new position, using both velocities.
    {
        let time_before_collision = h * collision_t;
        let time_after_collision = h - time_before_collision;

        body_state.position = {
            let position_at_collision =
                start_position + start_linear_velocity * time_before_collision;

            position_at_collision + body_state.velocity() * time_after_collision
        };
    }

    // Applies tangent impulse (if any) to new state.

    // Use contact point velocity BEFORE normal impulse (stored in
    // normal_impulse_data) because the friction logic expects the
    // contact to still be moving toward the plane
    let contact_point_velocity_before_normal_impulse = normal_impulse_data.contact_point_velocity;

    if let Some(tangent_impulse_data) = get_rigid_body_plane_friction_impulse(
        1.0 / body_state.inverse_mass,
        body_derivative,
        normal,
        contact_point_velocity_before_normal_impulse,
        normal_impulse_magnitude,
        material,
        true, // is_bounce_collision: use only dynamic friction
    ) {
        let tangent = tangent_impulse_data.tangent;
        let magnitude = tangent_impulse_data.magnitude;

        let tangent_impulse = tangent * magnitude;

        // Torque = r × impulse (must apply to the full impulse vector, not separately)
        let torque_from_friction = r_torque.cross(tangent_impulse);

        // Applies tangent impulse to linear momentum.
        body_state.linear_momentum += tangent_impulse;

        // Applies tangent impulse to angular momentum.
        body_state.angular_momentum += torque_from_friction;
    }
}
