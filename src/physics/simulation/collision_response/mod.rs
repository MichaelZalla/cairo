use crate::{animation::lerp, physics::material::PhysicsMaterial, vec::vec3::Vec3};

use super::rigid_body::{
    rigid_body_simulation_state::RigidBodySimulationState, RigidBodyCollisionResponse,
};

fn get_point_plane_outgoing_velocity(
    plane_normal: Vec3,
    material: &PhysicsMaterial,
    velocity: &Vec3,
) -> (Vec3, Vec3) {
    // Compute elasticity response (in the normal direction).

    let velocity_in_along_normal = plane_normal * velocity.dot(plane_normal);

    let velocity_out_along_normal = -velocity_in_along_normal * material.restitution;

    // Compute friction response (in the tangent direction).

    let velocity_in_along_tangent = velocity - velocity_in_along_normal;

    let loss = (material.dynamic_friction * velocity_in_along_normal.mag())
        .min(velocity_in_along_tangent.mag());

    let velocity_out_along_tangent = if velocity_in_along_tangent.is_zero() {
        velocity_in_along_tangent
    } else {
        velocity_in_along_tangent - velocity_in_along_tangent.as_normal() * loss
    };

    (velocity_out_along_normal, velocity_out_along_tangent)
}

pub fn resolve_point_plane_collision_approximate(
    plane_normal: Vec3,
    material: &PhysicsMaterial,
    end_position: &mut Vec3,
    end_velocity: &mut Vec3,
    penetration_depth: f32,
) {
    let (v_out_normal, v_out_tangent) =
        get_point_plane_outgoing_velocity(plane_normal, material, end_velocity);

    *end_velocity = v_out_normal + v_out_tangent;

    *end_position += {
        let bias = if v_out_normal.mag() < 0.05 {
            0.005
        } else {
            0.0
        };

        // Comptues a minimum displacement vector (accounting for restitution).

        plane_normal * ((penetration_depth * (1.0 + material.restitution)) + bias)
    };
}

pub fn resolve_vertex_face_collision(
    material: &PhysicsMaterial,
    normal: Vec3,
    barycentric: Vec3,
    point_mass: f32,
    point_velocity: &mut Vec3,
    v0_mass: f32,
    v0_velocity: &mut Vec3,
    v1_mass: f32,
    v1_velocity: &mut Vec3,
    v2_mass: f32,
    v2_velocity: &mut Vec3,
) {
    // Computes the absolute velocity of the point on the triangle where the
    // collision occurred, using barycentric weighting of vertex velocities.

    let (u, v, w) = (barycentric.x, barycentric.y, barycentric.z);

    let b_velocity_in = *v0_velocity * u + *v1_velocity * v + *v2_velocity * w;

    // Computes the effective mass of the point on the triangle where the
    // collision occurred, using barycentric weighting of vertex masses.

    // @NOTE The denominator is needed in order to conserve momentum.
    let denominator = u * u + v * v + w * w;

    let b_mass = (u * v0_mass + v * v1_mass + w * v2_mass) / denominator;

    // Computes the center of momentum of the vertex-face collision.

    let center_of_momentum =
        (*point_velocity * point_mass + b_velocity_in * b_mass) / (point_mass + b_mass);

    // Incoming point velocities relative to this center of momentum.

    let a_relative_velocity_in = *point_velocity - center_of_momentum;

    let b_relative_velocity_in = b_velocity_in - center_of_momentum;

    // Relative velocity updates for the colliding points.

    let a_relative_velocity_out = {
        let (v_out_normal, v_out_tangent) =
            get_point_plane_outgoing_velocity(normal, material, &a_relative_velocity_in);

        v_out_normal + v_out_tangent
    };

    let b_relative_velocity_out = {
        let (v_out_normal, v_out_tangent) =
            get_point_plane_outgoing_velocity(-normal, material, &b_relative_velocity_in);

        v_out_normal + v_out_tangent
    };

    // Absolute velocity updates needed for colliding points.

    let a_velocity_out = a_relative_velocity_out + center_of_momentum;
    let b_velocity_out = b_relative_velocity_out + center_of_momentum;

    // Applies the velocity update for the colliding vertex .

    *point_velocity = a_velocity_out;

    // Distributes the velocity update for the colliding face, amongst its
    // weighted vertices.

    let b_velocity_delta = b_velocity_out - b_velocity_in;

    let b_velocity_delta_prime = b_velocity_delta / denominator;

    // Δv_0 = u * Δv'
    *v0_velocity += b_velocity_delta_prime * u;
    // Δv_1 = v * Δv'
    *v1_velocity += b_velocity_delta_prime * v;
    // Δv_2 = w * Δv'
    *v2_velocity += b_velocity_delta_prime * w;
}

pub fn resolve_edge_edge_collision(
    material: &PhysicsMaterial,
    p1_mass: f32,
    p1_velocity: &mut Vec3,
    p2_mass: f32,
    p2_velocity: &mut Vec3,
    q1_mass: f32,
    q1_velocity: &mut Vec3,
    q2_mass: f32,
    q2_velocity: &mut Vec3,
    s: f32,
    t: f32,
    normal: Vec3,
) {
    // Treats q_a as intersecting a plane defined by the point p_a and the
    // normal (norm(q_a - p_a)).

    // Computes the absolute velocities of P and Q using linear interpolation.

    let p_velocity_in = lerp(*p1_velocity, *p2_velocity, s);
    let q_velocity_in = lerp(*q1_velocity, *q2_velocity, t);

    // Computes the effective mass of P and Q using a barycentric weighting of
    // the edges' vertex masses.

    let p_mass = {
        let (u, v) = (s, 1.0 - s);

        (p1_mass * u + p2_mass * v) / (u * u + v * v)
    };

    let q_mass = {
        let (u, v) = (t, 1.0 - t);

        (q1_mass * u + q2_mass * v) / (u * u + v * v)
    };

    // Computes a center-of-momentum for the edge-edge collision.

    let center_of_momentum = (p_velocity_in * p_mass + q_velocity_in * q_mass) / (p_mass + q_mass);

    // Incoming point velocities relative to this center of momentum.

    let p_relative_velocity_in = p_velocity_in - center_of_momentum;
    let q_relative_velocity_in = q_velocity_in - center_of_momentum;

    // Relative velocity updates for points P and Q.

    let p_relative_velocity_out = {
        let (v_out_normal, v_out_tangent) =
            get_point_plane_outgoing_velocity(normal, material, &p_relative_velocity_in);

        v_out_normal + v_out_tangent
    };

    let q_relative_velocity_out = {
        let (v_out_normal, v_out_tangent) =
            get_point_plane_outgoing_velocity(-normal, material, &q_relative_velocity_in);

        v_out_normal + v_out_tangent
    };

    // Absolute velocity updates needed for points P and Q.

    let p_velocity_out = p_relative_velocity_out + center_of_momentum;
    let q_velocity_out = q_relative_velocity_out + center_of_momentum;

    // Assume the velocities of point P before and after the collision are v_p-
    // and v_p+; since point P doesn't exist as its own mesh vertex, we need to
    // apply separate velocity updates to vertices P1 and P2, such that point P
    // receives the effective velocity update Δv = (v_p+ - v_p-).

    // The update to point P's velocity can then be expressed as:
    //
    //   Δv = u(Δv_1) + v(Δv_2)
    //
    // We can in turn define Δv_1 and Δv_2 as a single velocity update Δv',
    // weighted by u and v:
    //
    //   Δv_1 = u * Δv
    //   Δv_2 = v * Δv
    //
    // Therefore,
    //
    //   Δv = u(u * Δv') + v(v * Δv')
    //
    // By this equality,
    //
    //   Δv' = Δv / (u*u + v*v)
    //

    // Distributes the change in velocity among edge P's weighted vertices.

    let p_velocity_delta = p_velocity_out - p_velocity_in;

    let p_velocity_delta_prime = {
        let (u, v) = (s, 1.0 - s);

        p_velocity_delta / (u * u + v * v)
    };

    // Δv_1 = u * Δv'
    *p1_velocity += p_velocity_delta_prime * s;
    // Δv_2 = v * Δv'
    *p2_velocity += p_velocity_delta_prime * (1.0 - s);

    // Distributes the change in velocity among edge Q's weighted vertices.

    let q_velocity_delta = q_velocity_out - q_velocity_in;
    let q_velocity_delta_prime = {
        let (u, v) = (t, 1.0 - t);

        q_velocity_delta / (u * u + v * v)
    };

    // Δv_1 = u * Δv
    *q1_velocity += q_velocity_delta_prime * t;
    // Δv_2 = v * Δv
    *q2_velocity += q_velocity_delta_prime * (1.0 - t);
}

pub fn resolve_rigid_body_plane_collision(
    state: &mut RigidBodySimulationState,
    plane_normal: Vec3,
    contact_point: Vec3,
    contact_point_velocity: Vec3,
    r: Vec3,
    material: &PhysicsMaterial,
) -> RigidBodyCollisionResponse {
    let incoming_contact_point_speed_normal_to_plane = contact_point_velocity.dot(plane_normal);

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
    //               + (r_cross_n * I^-1).cross(r)
    //

    let numerator = -(1.0 + material.restitution) * incoming_contact_point_speed_normal_to_plane;

    let r_normal = r.as_normal();

    // A value of -1 indicates a rigid body moving directly into the plane; a
    // value of 1 indicates a body moving directly away from the plane; a value
    // of 0 means the body moves parallel to the plane.

    let r_dot_normal = r_normal.dot(plane_normal);

    if r_dot_normal.is_nan() {
        panic!()
    }

    let inverse_moment_of_intertia_world_space = state.inverse_moment_of_intertia_world_space();

    let change_in_angular_velocity_normalized = /* j * */
        r.cross(plane_normal) * inverse_moment_of_intertia_world_space;

    let change_in_angular_velocity_at_contact_point_normalized = /* j * */
        change_in_angular_velocity_normalized.cross(r);

    let denominator = state.inverse_mass
        + plane_normal.dot(change_in_angular_velocity_at_contact_point_normalized);

    let normal_impulse_magnitude = numerator / denominator;

    let normal_impulse = plane_normal * normal_impulse_magnitude;

    state.linear_momentum += normal_impulse;

    let rotation_axis = r.cross(plane_normal);

    state.angular_momentum += rotation_axis * normal_impulse_magnitude;

    RigidBodyCollisionResponse {
        contact_point,
        contact_point_velocity,
        normal_impulse,
    }
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
