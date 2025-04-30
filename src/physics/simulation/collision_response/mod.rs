use crate::{physics::material::PhysicsMaterial, vec::vec3::Vec3};

use super::rigid_body::rigid_body_simulation_state::RigidBodySimulationState;

pub fn resolve_point_plane_collision_approximate(
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

pub fn resolve_rigid_body_plane_collision(
    state: &mut RigidBodySimulationState,
    plane_normal: Vec3,
    contact_point: Vec3,
    material: &PhysicsMaterial,
) {
    let r = contact_point - state.position;

    let linear_velocity = state.velocity();

    let contact_point_velocity = linear_velocity + state.angular_velocity().cross(r);

    let incoming_contact_point_speed_normal_to_plane = contact_point_velocity.dot(plane_normal);

    let inverse_moment_of_intertia_world_space = state.inverse_moment_of_intertia_world_space();

    let change_in_angular_velocity_normalized = /* j * */
        r.cross(plane_normal) * inverse_moment_of_intertia_world_space;

    let change_in_angular_velocity_at_contact_point_normalized = /* j * */
        change_in_angular_velocity_normalized.cross(r);

    // v_outgoing = -v_incoming * restitution
    //
    // J = j * plane_normal
    //
    // Change in linear momentum P = J
    //
    // Change in linear momentum L = r.cross(J)
    //                             = j * r.cross(n)
    //
    // v_outgoing = (lv_incoming + lv_delta) + (av_incoming + av_delta).cross(r)
    //            = ln_incoming + av_incoming.cross(r) + lv_delta + av_delta.cross(r)
    //            = (incoming rate-of-change of contact point position) + lv_delta + av_delta.cross(r)
    //            = (incoming rate-of-change of contact point position)
    //               + (1/mass) * j * plane_normal
    //               + (r.cross(plane_normal) * I^-1).cross(r)
    //

    let numerator = -(1.0 + material.restitution) * incoming_contact_point_speed_normal_to_plane;

    let denominator = state.inverse_mass
        + plane_normal.dot(change_in_angular_velocity_at_contact_point_normalized);

    let normal_impulse_magnitude = numerator / denominator;

    state.linear_momentum += plane_normal * normal_impulse_magnitude;
    state.angular_momentum += r.cross(plane_normal) * normal_impulse_magnitude;
}

#[allow(unused_variables)]
pub fn resolve_rigid_body_collision(
    a: &mut RigidBodySimulationState,
    b: &mut RigidBodySimulationState,
    contact_point: Vec3,
    material: &PhysicsMaterial,
) {
    let r_a = contact_point - a.position;
    let r_b = contact_point - b.position;

    let incoming_contact_velocity_a = a.velocity() + a.angular_velocity().cross(r_a);
    let incoming_contact_velocity_b = b.velocity() + b.angular_velocity().cross(r_b);

    let normal = (a.position - b.position).as_normal();

    let incoming_speed_relative_to_normal =
        normal.dot(incoming_contact_velocity_a - incoming_contact_velocity_b);

    if incoming_speed_relative_to_normal > 0.0 {
        // Bodies are already moving away from each other.

        return;
    }

    // Change in angular velocity for rigid body A.

    let inverse_moment_of_intertia_a_world_space = a.inverse_moment_of_intertia_world_space();

    let change_in_angular_velocity_a_normalized = /* j * */
        r_a.cross(normal) * inverse_moment_of_intertia_a_world_space;

    let change_in_angular_velocity_at_contact_point_a_normalized = /* j * */
        change_in_angular_velocity_a_normalized.cross(r_a);

    // Change in angular velocity for rigid body B.

    let inverse_moment_of_intertia_b_world_space = b.inverse_moment_of_intertia_world_space();

    let change_in_angular_velocity_b_normalized = /* j * */
        r_b.cross(normal) * inverse_moment_of_intertia_b_world_space;

    let change_in_angular_velocity_at_contact_point_b_normalized = /* j * */
        change_in_angular_velocity_b_normalized.cross(r_b);

    // Calculate the normal impulse.

    let numerator = -(1.0 + material.restitution) * incoming_speed_relative_to_normal;

    let denominator = a.inverse_mass
        + b.inverse_mass
        + normal.dot(
            change_in_angular_velocity_at_contact_point_a_normalized
                + change_in_angular_velocity_at_contact_point_b_normalized,
        );

    let normal_impulse_magnitude = numerator / denominator;

    // Distribute the normal impulse to bodies.

    a.linear_momentum += normal * normal_impulse_magnitude;
    a.angular_momentum += r_a.cross(normal) * normal_impulse_magnitude;

    b.linear_momentum -= normal * normal_impulse_magnitude;
    b.angular_momentum -= r_b.cross(normal) * normal_impulse_magnitude;
}
