use std::f32::consts::PI;

use cairo::{
    color,
    geometry::{
        accelerator::{
            hash_grid::{GridSpaceCoordinate, HashGrid},
            static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::{intersect_capsule_plane, intersect_moving_spheres},
        primitives::sphere::Sphere,
    },
    matrix::Mat4,
    mesh::Mesh,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::{
                NormalImpulseData, get_rigid_body_plane_friction_impulse,
                get_rigid_body_plane_normal_impulse, resolve_rigid_body_collision,
            },
            contact::{StaticContact, StaticContactKind, StaticContactList},
            rigid_body::{
                RigidBody, RigidBodyKind,
                rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
            },
        },
    },
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::integration::system_dynamics_function;

use crate::{plane_collider::PlaneCollider, state_vector::StateVector};

/// Represents a collision surface with all necessary information for collision response.
struct CollisionSurface {
    /// Surface normal (pointing away from the surface)
    normal: Vec3,
    /// Contact point in world space
    contact_point: Vec3,
    /// Normalized time of collision [0, 1] within the timestep
    collision_t: f32,
    /// Minimum distance from body center to surface (e.g., sphere radius)
    minimum_distance: f32,
}

const CONTACT_DISTANCE_THRESHOLD: f32 = 0.005;

// Threshold for resting contact (both linear and contact point nearly zero)
const RESTING_SPEED_THRESHOLD: f32 = 0.05;

// Rolling detection: contact velocity should be small relative to body velocity
const ROLLING_VELOCITY_RATIO_THRESHOLD: f32 = 0.1; // Contact velocity < 10% of body velocity
const BODY_VELOCITY_ROLLING_THRESHOLD: f32 = 0.01; // Minimum body velocity for ratio-based rolling check
const CONTACT_POINT_VELOCITY_ROLLING_THRESHOLD: f32 = 0.2; // Absolute contact point velocity threshold at low body speeds

static RIGID_BODY_STATIC_PLANE_MATERIAL: PhysicsMaterial = PhysicsMaterial {
    static_friction: PI / 4.0,
    dynamic_friction: 0.1, // Low friction to preserve horizontal motion during bounces
    restitution: 0.4,
};

static RIGID_BODY_RIGID_BODY_MATERIAL: PhysicsMaterial = PhysicsMaterial {
    static_friction: PI / 4.0,
    dynamic_friction: 0.6,
    restitution: 0.4,
};

// Hash grid neighbor offsets for collision detection.
// These offsets check the current cell plus 13 neighboring cells in the "upper forward right"
// half-space. This avoids duplicate checks since each collision pair will be checked exactly once.
const HASH_GRID_NEIGHBORS: [(isize, isize, isize); 14] = [
    // Center
    (0, 0, 0),
    // Top far row
    (-1, 1, -1),
    (0, 1, -1),
    (1, 1, -1),
    // Top middle row
    (-1, 1, 0),
    (0, 1, 0),
    (1, 1, 0),
    // Top near row
    (-1, 1, 1),
    (0, 1, 1),
    (1, 1, 1),
    // Middle far right
    (1, 0, -1),
    // Middle right
    (1, 0, 0),
    // Middle near right
    (1, 0, 1),
    // Middle near
    (0, 0, 1),
];

pub struct Simulation {
    pub forces: Vec<Box<DynRigidBodyForce>>,
    pub rigid_bodies: Vec<RigidBody>,
    pub static_plane_colliders: Vec<PlaneCollider>,
    pub ground_mesh: Mesh,
    pub ground_tlas: StaticTriangleTLAS,
    pub hash_grid: HashGrid,
    pub visualize_hash_grid: bool,
    pub tick_count: u32,
}

impl Simulation {
    pub fn tick(&mut self, h: f32, uptime_seconds: f32) {
        self.tick_count += 1;

        // 1. Generate state vector from current rigid bodies
        let state = self.copy_to_state_vector();

        // 2. Predict contacts that will occur during this time step (Phase 1)
        let predicted_contacts = self.predict_contacts(&state, h);

        // 3. Compute derivatives using forces + predicted contacts (Phase 2)
        let derivative =
            system_dynamics_function(&state, &self.forces, &predicted_contacts, h, uptime_seconds);

        // 4. Integrate velocities (Phase 3)
        let mut state_with_new_velocities = self.integrate_velocities(&state, &derivative, h);

        // 5. Clear collision debug markers
        self.clear_collision_debug_info();

        // 6. Apply predictive impulses to prevent collisions (Phase 4)
        self.apply_predictive_impulses(&mut state_with_new_velocities, h, &derivative, &state);

        // 7. Integrate positions with corrected velocities (Phase 5)
        let mut final_state = self.integrate_positions(&state_with_new_velocities, h);

        // 8. Rebuild spatial acceleration structure
        self.rebuild_hash_grid(&final_state);

        // 9. (Safety net) Apply corrective impulses for any remaining collisions
        self.apply_corrective_impulses(h, &derivative, &state, &mut final_state);

        // 10. Copy final state back to simulation
        self.apply_state_vector(&final_state);
    }

    fn resolve_static_plane_collisions(
        &mut self,
        h: f32,
        derivative: &StateVector<RigidBodySimulationState>,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
        material: &PhysicsMaterial,
    ) {
        for (collider_index, collider) in self.static_plane_colliders.iter().enumerate() {
            for body_index in 0..self.rigid_bodies.len() {
                let body_derivative = &derivative.0[0];
                let current_body_state = &current_state.0[body_index];
                let new_body_state = &mut new_state.0[body_index];

                if collider_index == 0 {
                    // Clears any resting or sliding contacts from the last simulation tick.

                    new_body_state.static_contacts.clear();
                }

                Self::handle_static_plane_collision(
                    h,
                    body_derivative,
                    current_body_state,
                    new_body_state,
                    collider,
                    material,
                );
            }
        }
    }

    /// Removes linear momentum accumulated after the collision point.
    fn remove_post_collision_momentum(
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

    /// Corrects penetration by pushing the body out to maintain minimum distance.
    fn correct_static_plane_penetration(
        body_state: &mut RigidBodySimulationState,
        collider: &PlaneCollider,
        minimum_distance_to_plane: f32,
    ) {
        let signed_distance_from_body_to_plane =
            collider.plane.get_signed_distance(&body_state.position);

        if signed_distance_from_body_to_plane < minimum_distance_to_plane {
            let normal = collider.plane.normal;

            body_state.position +=
                normal * (minimum_distance_to_plane - signed_distance_from_body_to_plane);
        }
    }

    /// Handles static contact detection and momentum adjustment after
    /// collision.
    fn handle_static_contact_after_plane_collision(
        body_derivative: &RigidBodySimulationState,
        body_state: &mut RigidBodySimulationState,
        collider: &PlaneCollider,
        minimum_distance_to_plane: f32,
        material: &PhysicsMaterial,
    ) {
        if let Some(contact) = Self::get_static_contact_for_plane_collider(
            collider,
            body_derivative,
            body_state,
            minimum_distance_to_plane,
            material,
            true, // Post-integration contact detection
        ) {
            // Removes the component of linear momentum pointing into the
            // collider.
            body_state.linear_momentum -=
                collider.plane.normal * body_state.linear_momentum.dot(collider.plane.normal);

            // Allows spinning around the surface normal, but dampens angular
            // momentum that causes the contact point to slide.
            if matches!(contact.kind, StaticContactKind::Resting) {
                let normal = contact.normal;

                // Decomposes angular momentum into components parallel and
                // perpendicular to normal.
                let angular_momentum_along_normal =
                    normal * body_state.angular_momentum.dot(normal);

                let angular_momentum_perpendicular_to_normal =
                    body_state.angular_momentum - angular_momentum_along_normal;

                // Allows spinning around normal (like a top), but damp
                // tangential rotation.
                body_state.angular_momentum =
                    angular_momentum_along_normal + angular_momentum_perpendicular_to_normal * 0.95;
            }

            // Records the contact.
            body_state.static_contacts.push(contact).unwrap();
        }
    }

    /// Applies collision impulses (normal and friction) to the body state.
    ///
    /// This function:
    /// 1. Applies the normal impulse to linear and angular momentum
    /// 2. Updates the body's position based on pre/post-collision velocities
    /// 3. Calculates and applies friction impulse (if any)
    #[allow(clippy::too_many_arguments)]
    fn apply_collision_impulses(
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
        let contact_point_velocity_before_normal_impulse =
            normal_impulse_data.contact_point_velocity;

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

    /// Calculates collision response data if a collision is detected.
    ///
    /// Returns `Some((normal_impulse_data, collision_t))` if a collision occurred,
    /// where `collision_t` is the normalized time of collision within the timestep.
    /// Returns `None` if no collision occurred or if the contact point is not
    /// moving into the plane.
    fn calculate_collision_response(
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
            Self::remove_post_collision_momentum(
                h,
                surface.collision_t,
                body_derivative,
                new_body_state,
            );

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

    /// Returns `None` if no collision occurred or if the contact point is not
    /// moving into the plane.
    #[allow(clippy::too_many_arguments)]
    fn calculate_static_plane_collision_response(
        h: f32,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        collider: &PlaneCollider,
        minimum_distance_to_plane: f32,
        material: &PhysicsMaterial,
    ) -> Option<(NormalImpulseData, f32)> {
        let start_position = current_body_state.position;
        let end_position = new_body_state.position;
        let normal = collider.plane.normal;

        // Tests for a body-collider intersection, according to the kind of
        // rigid body.
        let intersection = match new_body_state.kind {
            RigidBodyKind::Sphere(radius) => {
                intersect_capsule_plane(start_position, end_position, radius, &collider.plane)
            }
            _ => panic!("Unsupported rigid body kind!"),
        };

        match intersection {
            Some(capsule_intersection) => {
                let surface = CollisionSurface {
                    normal,
                    contact_point: capsule_intersection.contact_point,
                    collision_t: capsule_intersection.t,
                    minimum_distance: minimum_distance_to_plane,
                };

                Self::calculate_collision_response(
                    h,
                    body_derivative,
                    new_body_state,
                    &surface,
                    material,
                )
            }
            None => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_static_plane_collision(
        h: f32,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        collider: &PlaneCollider,
        material: &PhysicsMaterial,
    ) {
        let start_position = current_body_state.position;
        let start_linear_velocity = current_body_state.velocity();

        let end_linear_velocity = new_body_state.velocity();

        // Tests whether any elastic or sliding collision may have occurred.

        let normal = collider.plane.normal;

        let body_speed_along_normal = end_linear_velocity.dot(normal);

        if body_speed_along_normal > RESTING_SPEED_THRESHOLD {
            // The rigid body is moving away from the plane, allow it to
            // continue on that trajectory.

            // @NOTE This may only hold true for spheres!

            return;
        }

        // Determines a minimum distance that this body must maintain.
        let minimum_distance_to_plane = match new_body_state.kind {
            RigidBodyKind::Sphere(radius) => radius,
            _ => panic!(),
        };

        // Calculate collision response data (if a collision occurred).
        let normal_impulse_data_with_t = Self::calculate_static_plane_collision_response(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            collider,
            minimum_distance_to_plane,
            material,
        );

        // Applies collision impulses (normal and friction) to new state.
        if let Some((normal_impulse_data, t)) = &normal_impulse_data_with_t {
            Self::apply_collision_impulses(
                h,
                body_derivative,
                new_body_state,
                start_position,
                start_linear_velocity,
                normal_impulse_data,
                *t,
                material,
            );
        }

        // If necessary, fixes up the body's position to maintain its minimum
        // distance from the collider.
        Self::correct_static_plane_penetration(new_body_state, collider, minimum_distance_to_plane);

        // Determines if the resulting body state constitutes a resting or
        // a sliding contact.
        Self::handle_static_contact_after_plane_collision(
            body_derivative,
            new_body_state,
            collider,
            minimum_distance_to_plane,
            material,
        );
    }

    fn compute_contact_point_velocity(
        body_velocity: Vec3,
        angular_velocity: Vec3,
        contact_point: Vec3,
        body_position: Vec3,
    ) -> Vec3 {
        let r = contact_point - body_position;

        body_velocity + angular_velocity.cross(r)
    }

    fn compute_motion_tangent(
        point_on_plane: Vec3,
        velocity: Vec3,
        plane: &cairo::geometry::primitives::plane::Plane,
    ) -> Option<Vec3> {
        let v = point_on_plane + velocity;
        let d = plane.get_signed_distance(&v);
        let velocity_projected_onto_plane = v - plane.normal * d;

        if velocity_projected_onto_plane.is_zero() {
            None
        } else {
            Some(velocity_projected_onto_plane.as_normal())
        }
    }

    fn classify_contact_kind(
        body_velocity: Vec3,
        contact_point_velocity: Vec3,
        body_motion_tangent: Option<Vec3>,
        contact_point_motion_tangent: Option<Vec3>,
        angular_velocity: Vec3,
        normal: Vec3,
    ) -> StaticContactKind {
        // Constants for motion/spin significance thresholds
        const MIN_VELOCITY_FOR_AXIS_CHECK: f32 = 0.01;

        const MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK: f32 = 0.01;

        const MIN_VELOCITY_FOR_AXIS_CHECK_SQUARED: f32 =
            MIN_VELOCITY_FOR_AXIS_CHECK * MIN_VELOCITY_FOR_AXIS_CHECK;

        const MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK_SQUARED: f32 =
            MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK * MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK;

        let body_speed_along_tangent = if let Some(tangent) = body_motion_tangent {
            body_velocity.dot(tangent)
        } else {
            0.0
        };

        let contact_point_speed_along_tangent = if let Some(tangent) = contact_point_motion_tangent
        {
            contact_point_velocity.dot(tangent)
        } else {
            0.0
        };

        // Early return for resting: if both velocities are very small, body is at rest.
        // Note that a body with very low tangential velocity but high Y-spin will be
        // classified as "resting" here (spinning in place).
        if body_speed_along_tangent < RESTING_SPEED_THRESHOLD
            && contact_point_speed_along_tangent < RESTING_SPEED_THRESHOLD
        {
            return StaticContactKind::Resting;
        }

        // Check if contact velocity is small relative to body velocity (rolling)
        let velocity_ratio = if body_speed_along_tangent > BODY_VELOCITY_ROLLING_THRESHOLD {
            contact_point_speed_along_tangent / body_speed_along_tangent
        } else {
            // At very low speeds, avoid division by near-zero; use absolute threshold instead
            if contact_point_speed_along_tangent < CONTACT_POINT_VELOCITY_ROLLING_THRESHOLD {
                0.0 // Treat as rolling
            } else {
                1.0 // Treat as sliding
            }
        };

        let is_rolling = velocity_ratio < ROLLING_VELOCITY_RATIO_THRESHOLD;

        // Additional check for rolling: the spin axis must be orthogonal to
        // both (1) the surface normal, and (2) the direction of motion.

        let has_correct_spin_axis = {
            let has_significant_motion =
                body_velocity.mag_squared() > MIN_VELOCITY_FOR_AXIS_CHECK_SQUARED;

            let has_significant_spin =
                angular_velocity.mag_squared() > MIN_ANGULAR_VELOCITY_FOR_AXIS_CHECK_SQUARED;

            if has_significant_motion && has_significant_spin {
                let spin_axis = angular_velocity.as_normal();
                let motion_direction = body_velocity.as_normal();

                // Spin must be perpendicular to normal.
                let dot_with_normal = normal.dot(spin_axis).abs();
                let perpendicular_to_normal = dot_with_normal < 0.1;

                // Spin must be perpendicular to motion.
                let dot_with_motion = motion_direction.dot(spin_axis);
                let perpendicular_to_motion = dot_with_motion.abs() < 0.1;

                perpendicular_to_motion && perpendicular_to_normal
            } else if has_significant_spin {
                // High spin but low tangential velocity: check if spin is parallel
                // to surface normal (i.e., Y-spin); if so, this is definitely NOT
                // rolling.
                let spin_axis = angular_velocity.as_normal();
                let dot_with_normal = normal.dot(spin_axis).abs();
                let parallel_to_normal = dot_with_normal > 0.9;

                !parallel_to_normal
            } else {
                // Low velocity AND low spin - can't determine axis, allow resting/rolling
                true
            }
        };

        if is_rolling && has_correct_spin_axis {
            StaticContactKind::Rolling(0.5)
        } else {
            StaticContactKind::Sliding
        }
    }

    fn compute_contact_basis(
        normal: Vec3,
        contact_point_motion_tangent: Option<Vec3>,
    ) -> (Vec3, Vec3) {
        match contact_point_motion_tangent {
            Some(tangent) => (tangent, tangent.cross(normal)),
            None => {
                let (_, tangent, bitangent) = normal.basis();
                (tangent, bitangent)
            }
        }
    }

    /// Generic static contact detection for any surface (plane or triangle mesh).
    #[allow(clippy::too_many_arguments)]
    fn get_static_contact(
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        surface_normal: Vec3,
        signed_distance: f32,
        minimum_distance: f32,
        body_motion_tangent: Option<Vec3>,
        contact_point_motion_tangent: Option<Vec3>,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> Option<StaticContact> {
        // 1. Requires that the body be very close to the collider
        //    (within some threshold).

        if signed_distance > minimum_distance + CONTACT_DISTANCE_THRESHOLD {
            return None;
        }

        // 2. Requires that the contact point's speed along the normal be very small
        //    (within some threshold).

        let body_velocity = new_body_state.velocity();

        let contact_point = new_body_state.position - surface_normal * minimum_distance;

        let contact_point_velocity = Self::compute_contact_point_velocity(
            body_velocity,
            new_body_state.angular_velocity(),
            contact_point,
            new_body_state.position,
        );

        let contact_point_speed_along_normal = contact_point_velocity.dot(surface_normal);

        if contact_point_speed_along_normal.abs() > RESTING_SPEED_THRESHOLD {
            return None;
        }

        // 3. (Optional) Requires that the forces acting on the body aren't
        //    pulling the body away from the collider.

        if check_forces {
            let magnitude_of_acceleration_along_normal =
                derivative.linear_momentum.dot(surface_normal);

            // @NOTE For a rigid body shape that is more complex than a sphere, the
            // body may be moving away from the plane, while the point of contact is
            // moving into it!

            if magnitude_of_acceleration_along_normal > f32::EPSILON {
                return None;
            }
        }

        // At this point, we can be certain that the object is resting, rolling
        // along the plane, or sliding along the plane.

        let angular_velocity = new_body_state.angular_velocity();

        let kind = Self::classify_contact_kind(
            body_velocity,
            contact_point_velocity,
            body_motion_tangent,
            contact_point_motion_tangent,
            angular_velocity,
            surface_normal,
        );

        let (tangent, bitangent) =
            Self::compute_contact_basis(surface_normal, contact_point_motion_tangent);

        Some(StaticContact {
            kind,
            point: contact_point,
            normal: surface_normal,
            tangent,
            bitangent,
            contact_point_velocity,
            material: *material,
        })
    }

    /// Plane-specific static contact detection.
    fn get_static_contact_for_plane_collider(
        collider: &PlaneCollider,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        radius: f32,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> Option<StaticContact> {
        let normal = collider.plane.normal;

        let signed_distance_to_plane = collider.plane.get_signed_distance(&new_body_state.position);

        let body_velocity = new_body_state.velocity();

        let contact_point = new_body_state.position - normal * radius;

        let contact_point_velocity = Self::compute_contact_point_velocity(
            body_velocity,
            new_body_state.angular_velocity(),
            contact_point,
            new_body_state.position,
        );

        let body_motion_tangent =
            Self::compute_motion_tangent(collider.point, body_velocity, &collider.plane);

        let contact_point_motion_tangent =
            Self::compute_motion_tangent(collider.point, contact_point_velocity, &collider.plane);

        Self::get_static_contact(
            derivative,
            new_body_state,
            normal,
            signed_distance_to_plane,
            radius,
            body_motion_tangent,
            contact_point_motion_tangent,
            material,
            check_forces,
        )
    }

    fn resolve_rigid_body_collisions(
        &mut self,
        h: f32,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
        material: &PhysicsMaterial,
    ) {
        for current_body_index in 0..self.rigid_bodies.len() {
            let (strategy, scale) = (self.hash_grid.strategy, self.hash_grid.scale);

            let body_coordinate = {
                let body = &new_state.0[current_body_index];

                GridSpaceCoordinate::from((body, strategy, scale))
            };

            for (x_offset, y_offset, z_offset) in HASH_GRID_NEIGHBORS.iter() {
                // Checks current cell and neighborings cells.

                let offset = GridSpaceCoordinate {
                    x: *x_offset,
                    y: *y_offset,
                    z: *z_offset,
                };

                let offset_coordinate = body_coordinate + offset;

                if let Some(cell) = self.hash_grid.map.get(&offset_coordinate) {
                    for neighbor_body_index in cell {
                        if *neighbor_body_index == current_body_index {
                            // Avoids a test for self-collision.

                            continue;
                        }

                        if Simulation::did_resolve_rigid_bodies_collision(
                            h,
                            current_state,
                            new_state,
                            current_body_index,
                            *neighbor_body_index,
                            material,
                        ) {
                            // sphere.collision_impulse.replace(...);
                        }
                    }
                }
            }
        }
    }

    fn did_resolve_rigid_bodies_collision(
        h: f32,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
        a: usize,
        b: usize,
        material: &PhysicsMaterial,
    ) -> bool {
        // Describes the movement of body A from body B's frame of reference.

        let state_a = &current_state.0[a];
        let state_b = &current_state.0[b];

        let a_velocity_relative_to_b = {
            let velocity_a = new_state.0[a].velocity();
            let velocity_b = new_state.0[b].velocity();

            velocity_a - velocity_b
        };

        let a_velocity_relative_to_b_mag = a_velocity_relative_to_b.mag();

        if a_velocity_relative_to_b_mag.abs() < f32::EPSILON {
            return false;
        }

        // Narrow-phase collision test.

        let intersection = match (state_a.kind, state_b.kind) {
            (RigidBodyKind::Sphere(radius_a), RigidBodyKind::Sphere(radius_b)) => {
                // Sphere-sphere collision.

                let s1 = Sphere {
                    center: current_state.0[a].position,
                    radius: radius_a,
                };

                let s2 = Sphere {
                    center: current_state.0[b].position,
                    radius: radius_b,
                };

                intersect_moving_spheres(
                    s1,
                    s2,
                    a_velocity_relative_to_b,
                    a_velocity_relative_to_b_mag,
                )
            }
            _ => panic!(
                "Collision not supported for rigid body pair {}, {}!",
                state_a.kind, state_b.kind
            ),
        };

        match intersection {
            Some(collision) => {
                // Normalizes distance from distance units to [0,1] (timestep fraction).
                // Distance traveled this frame = a_velocity_relative_to_b_mag * h
                // Fraction of frame = collision.distance / (a_velocity_relative_to_b_mag * h)
                let t_normalized = collision.distance / (a_velocity_relative_to_b_mag * h);

                // Only handle collisions that occur within this timestep (t_normalized <= 1.0)
                if t_normalized > 1.0 {
                    return false;
                }

                // Compute and apply the collision response.

                let mut s1_state_cloned = new_state.0[a];
                let mut s2_state_cloned = new_state.0[b];

                resolve_rigid_body_collision(
                    &mut s1_state_cloned,
                    &mut s2_state_cloned,
                    collision.contact_point,
                    material,
                );

                new_state.0[a].linear_momentum = s1_state_cloned.linear_momentum;
                new_state.0[a].angular_momentum = s1_state_cloned.angular_momentum;

                new_state.0[b].linear_momentum = s2_state_cloned.linear_momentum;
                new_state.0[b].angular_momentum = s2_state_cloned.angular_momentum;

                true
            }
            None => false,
        }
    }

    fn rebuild_hash_grid(&mut self, new_state: &StateVector<RigidBodySimulationState>) {
        self.hash_grid.map.clear();

        for (i, sphere_state) in new_state.0.iter().enumerate() {
            let (strategy, scale) = (self.hash_grid.strategy, self.hash_grid.scale);

            let coord = GridSpaceCoordinate::from((sphere_state, strategy, scale));

            match self.hash_grid.map.get_mut(&coord) {
                Some(list) => {
                    list.push(i);
                }
                None => {
                    let mut cell = Vec::<usize>::with_capacity(4);

                    cell.insert(0, i);

                    self.hash_grid.map.insert(coord, cell);
                }
            }
        }
    }

    /// Renders the hash grid visualization as semi-transparent cubes.
    fn render_hash_grid(&self, renderer: &mut SoftwareRenderer) {
        if !self.visualize_hash_grid {
            return;
        }

        for grid_coord in self.hash_grid.map.keys() {
            let transform = {
                let scale = Mat4::scale_uniform(0.5 * self.hash_grid.scale);
                let offset = Mat4::translation(vec3::ONES * 0.5);
                let translate = Mat4::translation(Vec3 {
                    x: (grid_coord.x as f32 * self.hash_grid.scale),
                    y: (grid_coord.y as f32 * self.hash_grid.scale),
                    z: (grid_coord.z as f32 * self.hash_grid.scale),
                });

                scale * offset * translate
            };

            let display_kind = EmptyDisplayKind::Cube;
            let color = Some(color::LIGHT_GRAY);

            renderer.render_empty(&transform, display_kind, false, color);
        }
    }

    /// Renders a static plane collider as a square with a normal vector.
    fn render_static_plane_collider(collider: &PlaneCollider, renderer: &mut SoftwareRenderer) {
        // Calculate the coordinate frame for the plane
        let mut right = collider.plane.normal.cross(vec3::UP);

        if right.mag() < f32::EPSILON {
            right = collider.plane.normal.cross(vec3::FORWARD);
        }

        right = right.as_normal();

        let up = collider.plane.normal.cross(-right);

        // Render the plane as an orange square
        let scale = Mat4::scale_uniform(20.0);
        let translate = Mat4::translation(collider.point);
        let rotate = Mat4::tbn(right, up, collider.plane.normal);

        let transform = scale * rotate * translate;

        renderer.render_empty(
            &transform,
            EmptyDisplayKind::Square,
            false,
            Some(color::ORANGE),
        );

        // Render the normal vector as a green line
        renderer.render_line(
            collider.point,
            collider.point + collider.plane.normal * 5.0,
            color::GREEN,
        );
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        renderer.render_axes(None);

        // Render dynamic rigid bodies (spheres).
        for sphere in &self.rigid_bodies {
            sphere.render(renderer);
        }

        // Visualize hash grid entries.
        self.render_hash_grid(renderer);

        // Visualize static plane colliders.
        for collider in &self.static_plane_colliders {
            Self::render_static_plane_collider(collider, renderer);
        }

        // Render the ground mesh
        renderer.render_entity(&Mat4::identity(), &self.ground_mesh, &None);

        // Visualize the ground mesh BVH.
        if let Some(collider) = self.ground_mesh.collider.as_ref() {
            renderer.render_bvh(collider, 2);
        }
    }

    fn copy_to_state_vector(&self) -> StateVector<RigidBodySimulationState> {
        self.rigid_bodies.as_slice().into()
    }

    fn apply_state_vector(&mut self, state: &StateVector<RigidBodySimulationState>) {
        for (i, sphere) in self.rigid_bodies.iter_mut().enumerate() {
            sphere.apply_simulation_state(&state.0[i]);
        }
    }

    fn clear_collision_debug_info(&mut self) {
        for sphere in self.rigid_bodies.iter_mut() {
            sphere.collision_impulse.take();
        }
    }

    fn apply_corrective_impulses(
        &mut self,
        h: f32,
        derivative: &StateVector<RigidBodySimulationState>,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
    ) {
        // Applies impulses based on actual collisions at final positions.

        self.resolve_static_plane_collisions(
            h,
            derivative,
            current_state,
            new_state,
            &RIGID_BODY_STATIC_PLANE_MATERIAL,
        );

        self.resolve_rigid_body_collisions(
            h,
            current_state,
            new_state,
            &RIGID_BODY_RIGID_BODY_MATERIAL,
        );
    }

    fn apply_predictive_impulses(
        &mut self,
        state_with_new_velocities: &mut StateVector<RigidBodySimulationState>,
        h: f32,
        derivative: &StateVector<RigidBodySimulationState>,
        original_state: &StateVector<RigidBodySimulationState>,
    ) {
        // Phase 4: Apply impulses based on predicted collisions BEFORE position integration

        // 1. Create a predicted state by integrating positions from new velocities
        let mut predicted_state = self.integrate_positions(state_with_new_velocities, h);

        // 2. Rebuild hash grid with predicted positions for accurate collision detection
        self.rebuild_hash_grid(&predicted_state);

        // 3. Detect and resolve collisions at those predicted positions
        //    (This modifies predicted_state's velocities with impulses)
        self.apply_corrective_impulses(h, derivative, original_state, &mut predicted_state);

        // 4. Copy the velocity corrections back (but NOT position changes)
        for (i, body) in state_with_new_velocities.0.iter_mut().enumerate() {
            body.linear_momentum = predicted_state.0[i].linear_momentum;
            body.angular_momentum = predicted_state.0[i].angular_momentum;
        }
    }

    fn predict_contacts(
        &self,
        state: &StateVector<RigidBodySimulationState>,
        h: f32,
    ) -> Vec<StaticContactList<6>> {
        let mut predicted_contacts = vec![StaticContactList::default(); self.rigid_bodies.len()];

        for (body_index, body_state) in state.0.iter().enumerate() {
            let body = &self.rigid_bodies[body_index];

            // Get radius (minimum distance to surface)
            let radius = match body.kind {
                RigidBodyKind::Sphere(r) => r,
                _ => continue, // Skip unsupported body types
            };

            // Extrapolate position forward by timestep h
            let predicted_position = body_state.position + body_state.velocity() * h;

            // Create a predicted body state with the extrapolated position
            let mut predicted_body_state = *body_state;

            predicted_body_state.position = predicted_position;

            // Test against each static plane collider
            for collider in &self.static_plane_colliders {
                if let Some(contact) = Self::get_static_contact_for_plane_collider(
                    collider,
                    body_state, // Use current state for derivative (unused when check_forces=false)
                    &predicted_body_state,
                    radius,
                    &RIGID_BODY_STATIC_PLANE_MATERIAL,
                    false, // check_forces: false for prediction
                ) {
                    predicted_contacts[body_index].push(contact).ok();
                }
            }
        }

        predicted_contacts
    }

    fn integrate_velocities(
        &self,
        state: &StateVector<RigidBodySimulationState>,
        derivative: &StateVector<RigidBodySimulationState>,
        h: f32,
    ) -> StateVector<RigidBodySimulationState> {
        let mut new_state = state.clone();

        for (body, derivative) in new_state.0.iter_mut().zip(derivative.0.iter()) {
            // Update linear momentum from forces.
            body.linear_momentum += derivative.linear_momentum * h;

            // Update angular momentum from torques.
            body.angular_momentum += derivative.angular_momentum * h;
        }

        new_state
    }

    fn integrate_positions(
        &self,
        state: &StateVector<RigidBodySimulationState>,
        h: f32,
    ) -> StateVector<RigidBodySimulationState> {
        let mut new_state = state.clone();

        for body in new_state.0.iter_mut() {
            // Update position from linear momentum.
            body.position += body.linear_momentum * body.inverse_mass * h;

            // Compute angular velocity from angular momentum.
            let new_angular_spin = {
                let angular_momentum = Vec4::vector(body.angular_momentum);

                let inverse_moment_of_inertia_world_space =
                    body.inverse_moment_of_inertia_world_space();

                let angular_velocity =
                    (angular_momentum * inverse_moment_of_inertia_world_space).to_vec3();

                Quaternion::from_raw(0.0, angular_velocity)
            };

            // Compute orientation derivative from angular velocity.
            let orientation_derivative = body.orientation * 0.5 * new_angular_spin;

            // Update orientation.
            body.orientation += orientation_derivative * h;

            // Renormalize to maintain unit quaternion constraint
            body.orientation.renormalize();
        }

        new_state
    }
}
