use crate::{
    geometry::{
        accelerator::{
            static_triangle_bvh::StaticBVHCollision, static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::medial_primitive::intersect_medial_primitive_tlas,
        primitives::{capsule::Capsule, triangle::Triangle},
    },
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::{self, CollisionSurface, StaticCollider},
            contact::{self, StaticContactList},
            rigid_body::{RigidBodyKind, rigid_body_simulation_state::RigidBodySimulationState},
        },
    },
    vec::vec3::Vec3,
};

/// Helper function to extract triangle and world-space normal from a BVH
/// collision.
///
/// Returns (triangle, world_normal) for the colliding triangle.
fn get_collision_triangle_and_normal<'a>(
    tlas: &'a StaticTriangleTLAS,
    collision: &StaticBVHCollision,
) -> (&'a Triangle, Vec3) {
    let bvh_instance_index = collision.colliding_bvh_index;
    let tri_index = collision.colliding_primitive;
    let bvh_instance = &tlas.bvh_instances[bvh_instance_index];
    let triangle = &bvh_instance.bvh.tris[tri_index];

    // Get triangle normal in world space
    let triangle_normal = triangle.plane.normal;
    let normal = (triangle_normal * bvh_instance.transform).as_normal();

    (triangle, normal)
}

impl StaticCollider for StaticTriangleTLAS {
    fn detect_static_contacts(
        &self,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        radius: f32,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> StaticContactList<6> {
        let position = new_body_state.position;

        // Create a zero-length capsule at the position (sphere as degenerate capsule)
        let mut capsule = Capsule::new(position, position, radius);

        // Intersect with mesh TLAS to find all collisions
        intersect_medial_primitive_tlas(&mut capsule, self);

        let mut contacts = contact::StaticContactList::default();

        // Process all detected collisions
        for collision in capsule.segment.collisions.iter() {
            let (triangle, normal) = get_collision_triangle_and_normal(self, collision);

            // Calculate signed distance to triangle plane
            let signed_distance = triangle.plane.get_signed_distance(&position);

            // Calculate motion tangents
            let body_velocity = new_body_state.velocity();

            let contact_point = position - normal * radius;

            let contact_point_velocity = contact::compute_contact_point_velocity(
                body_velocity,
                new_body_state.angular_velocity(),
                contact_point,
                position,
            );

            let body_motion_tangent = {
                let velocity_in_plane = body_velocity - normal * body_velocity.dot(normal);
                if velocity_in_plane.mag_squared() > f32::EPSILON {
                    Some(velocity_in_plane.as_normal())
                } else {
                    None
                }
            };

            let contact_point_motion_tangent = {
                let velocity_in_plane =
                    contact_point_velocity - normal * contact_point_velocity.dot(normal);
                if velocity_in_plane.mag_squared() > f32::EPSILON {
                    Some(velocity_in_plane.as_normal())
                } else {
                    None
                }
            };

            if let Some(contact) = contact::get_static_contact(
                derivative,
                new_body_state,
                normal,
                signed_distance,
                radius,
                body_motion_tangent,
                contact_point_motion_tangent,
                material,
                check_forces,
            ) {
                contacts.push(contact).ok();
            }
        }

        contacts
    }

    fn detect_collision(
        &self,
        start_position: Vec3,
        end_position: Vec3,
        radius: f32,
    ) -> Option<CollisionSurface> {
        // Create capsule from start to end position
        let mut capsule = Capsule::new(start_position, end_position, radius);

        // Intersect capsule with mesh TLAS
        intersect_medial_primitive_tlas(&mut capsule, self);

        // Check if we hit a triangle
        if let Some(collision) = capsule.segment.closest_collision() {
            let (_triangle, normal) = get_collision_triangle_and_normal(self, collision);

            // Calculate collision point
            let t = collision.t;
            let collision_point = start_position + (end_position - start_position) * t;

            Some(CollisionSurface {
                normal,
                contact_point: collision_point,
                collision_t: t,
                minimum_distance: radius,
            })
        } else {
            None
        }
    }

    fn handle_collisions(
        &self,
        h: f32,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        material: &PhysicsMaterial,
    ) {
        // Get sphere radius
        let radius = match new_body_state.kind {
            RigidBodyKind::Sphere(r) => r,
            _ => return,
        };

        let start_position = current_body_state.position;
        let start_linear_velocity = current_body_state.velocity();

        // Calculate collision response data for all simultaneous collisions
        let collision_responses = calculate_mesh_collision_responses(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            self,
            radius,
            material,
        );

        // Apply collision impulses for all simultaneous collisions
        for (normal_impulse_data, t) in &collision_responses {
            collision::apply_collision_impulses(
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

        // If necessary, fixes up the body's position to maintain its minimum distance.
        correct_mesh_penetration(new_body_state, self, radius);

        // Determines if the resulting body state constitutes a resting or sliding contact.
        handle_static_contact_after_mesh_collision(
            body_derivative,
            new_body_state,
            self,
            radius,
            material,
        );
    }
}

fn calculate_mesh_collision_responses(
    h: f32,
    body_derivative: &RigidBodySimulationState,
    current_body_state: &RigidBodySimulationState,
    new_body_state: &mut RigidBodySimulationState,
    tlas: &StaticTriangleTLAS,
    radius: f32,
    material: &PhysicsMaterial,
) -> Vec<(collision::NormalImpulseData, f32)> {
    let start_position = current_body_state.position;
    let end_position = new_body_state.position;

    // Create capsule from start to end position
    let mut capsule = Capsule::new(start_position, end_position, radius);

    // Intersect capsule with mesh TLAS
    intersect_medial_primitive_tlas(&mut capsule, tlas);

    let mut collision_responses = vec![];

    // Find the earliest collision time
    let earliest_t = capsule.segment.closest_collision().map(|c| c.t);

    if let Some(earliest_t) = earliest_t {
        // Process all collisions that occur at the earliest time
        for collision in capsule.segment.collisions.iter() {
            // Only process collisions at the earliest t (within epsilon for floating point comparison)
            if (collision.t - earliest_t).abs() < f32::EPSILON {
                let (_triangle, normal) = get_collision_triangle_and_normal(tlas, collision);

                // Calculate collision point
                let collision_point =
                    start_position + (end_position - start_position) * collision.t;

                let surface = CollisionSurface {
                    normal,
                    contact_point: collision_point,
                    collision_t: collision.t,
                    minimum_distance: radius,
                };

                if let Some(response) = collision::calculate_collision_response(
                    h,
                    body_derivative,
                    new_body_state,
                    &surface,
                    material,
                ) {
                    collision_responses.push(response);
                }
            }
        }
    }

    collision_responses
}

fn correct_mesh_penetration(
    body_state: &mut RigidBodySimulationState,
    tlas: &StaticTriangleTLAS,
    radius: f32,
) {
    let position = body_state.position;

    // Create a zero-length capsule at the position
    let mut capsule = Capsule::new(position, position, radius);

    // Intersect with mesh TLAS
    intersect_medial_primitive_tlas(&mut capsule, tlas);

    // Process all detected collisions to correct penetration
    for collision in capsule.segment.collisions.iter() {
        let (triangle, normal) = get_collision_triangle_and_normal(tlas, collision);

        // Calculate signed distance from body to triangle plane
        let signed_distance_from_body_to_plane =
            triangle.plane.get_signed_distance(&body_state.position);

        // If penetrating, push the body out
        if signed_distance_from_body_to_plane < radius {
            body_state.position += normal * (radius - signed_distance_from_body_to_plane);
        }
    }
}

fn handle_static_contact_after_mesh_collision(
    body_derivative: &RigidBodySimulationState,
    body_state: &mut RigidBodySimulationState,
    tlas: &StaticTriangleTLAS,
    radius: f32,
    material: &PhysicsMaterial,
) {
    // Detect all static contacts
    let contacts = tlas.detect_static_contacts(
        body_derivative,
        body_state,
        radius,
        material,
        true, // Post-integration contact detection
    );

    // Process all detected contacts
    for contact in contacts.into_iter() {
        // Store the contact for later friction/torque application
        body_state.static_contacts.push(contact).ok();

        // Remove the component of linear momentum pointing into the surface
        let normal = contact.normal;
        body_state.linear_momentum -= normal * body_state.linear_momentum.dot(normal);
    }
}
