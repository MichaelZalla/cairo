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
            collision::{self, CollisionResponse, CollisionSurface, StaticCollider},
            contact::{self, StaticContactList},
            rigid_body::rigid_body_simulation_state::RigidBodySimulationState,
        },
    },
    vec::vec3::{self, Vec3},
};

/// Result of analyzing a BVH collision against a triangle.
struct TriangleCollisionData<'a> {
    /// The triangle that was hit.
    triangle: &'a Triangle,
    /// The triangle normal in world space.
    triangle_normal_world: Vec3,
}

/// Helper function to extract triangle and world-space normal from a BVH
/// collision.
///
/// Returns (triangle, world_normal) for the colliding triangle.
fn get_triangle_collision_data<'a>(
    tlas: &'a StaticTriangleTLAS,
    collision: &StaticBVHCollision,
) -> TriangleCollisionData<'a> {
    let bvh_instance_index = collision.colliding_bvh_index;
    let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

    let triangle_index = collision.colliding_primitive;
    let triangle = &bvh_instance.bvh.tris[triangle_index];

    // Get triangle normal in world space
    let collision_normal_local_space = triangle.plane.normal;

    let collision_normal_world_space =
        (collision_normal_local_space * bvh_instance.transform).as_normal();

    TriangleCollisionData {
        triangle,
        triangle_normal_world: collision_normal_world_space,
    }
}

impl StaticCollider for StaticTriangleTLAS {
    fn detect_static_contacts(
        &self,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> StaticContactList<6> {
        let position = new_body_state.position;

        let radius = new_body_state.radius();

        // Create a zero-length capsule at the position (sphere as degenerate capsule)
        let mut capsule = Capsule::new(position, position, radius);

        // Intersect with mesh TLAS to find all collisions
        intersect_medial_primitive_tlas(&mut capsule, self);

        let mut contacts = contact::StaticContactList::default();

        // Process all detected collisions
        for collision in capsule.segment.collisions.iter() {
            let collision_data = get_triangle_collision_data(self, collision);

            let triangle = collision_data.triangle;
            let normal = collision_data.triangle_normal_world;

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

    fn detect_collisions(
        &self,
        start_position: Vec3,
        end_position: Vec3,
        body_state: &RigidBodySimulationState,
    ) -> Vec<CollisionSurface> {
        let radius = body_state.radius();

        // Create capsule from start to end position
        let mut capsule = Capsule::new(start_position, end_position, radius);

        // Intersect capsule with mesh TLAS
        intersect_medial_primitive_tlas(&mut capsule, self);

        let mut surfaces = vec![];

        // Find the earliest collision time
        let earliest_t = capsule.segment.closest_collision().map(|c| c.t);

        if let Some(earliest_t) = earliest_t {
            // Process all collisions that occur at the earliest time
            for collision in capsule.segment.collisions.iter() {
                // Only process collisions at the earliest t (within epsilon for floating point comparison)
                if (collision.t - earliest_t).abs() < f32::EPSILON {
                    let collision_data = get_triangle_collision_data(self, collision);

                    let normal = collision_data.triangle_normal_world;

                    // Calculate collision point
                    let collision_point =
                        start_position + (end_position - start_position) * collision.t;

                    surfaces.push(CollisionSurface {
                        normal,
                        contact_point: collision_point,
                        collision_t: collision.t,
                        minimum_distance: radius,
                    });
                }
            }
        }

        surfaces
    }

    fn handle_collisions(
        &self,
        h: f32,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        material: &PhysicsMaterial,
    ) {
        let radius = new_body_state.radius();

        let start_position = current_body_state.position;
        let start_linear_velocity = current_body_state.velocity();

        // Calculate collision response data for all simultaneous collisions
        let collision_responses = calculate_mesh_collision_responses(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            self,
            material,
        );

        // Apply collision impulses for all simultaneous collisions
        for response in &collision_responses {
            collision::apply_collision_impulses(
                h,
                body_derivative,
                new_body_state,
                start_position,
                start_linear_velocity,
                &response.impulse_data,
                response.collision_time,
                material,
            );
        }

        // If necessary, fixes up the body's position to maintain its minimum distance.
        correct_mesh_penetration(new_body_state, self, radius);

        // Determines if the resulting body state constitutes a resting or sliding contact.
        handle_static_contact_after_mesh_collision(body_derivative, new_body_state, self, material);
    }
}

fn calculate_mesh_collision_responses(
    h: f32,
    body_derivative: &RigidBodySimulationState,
    current_body_state: &RigidBodySimulationState,
    new_body_state: &mut RigidBodySimulationState,
    tlas: &StaticTriangleTLAS,
    material: &PhysicsMaterial,
) -> Vec<CollisionResponse> {
    let start_position = current_body_state.position;
    let end_position = new_body_state.position;

    // Detect all collisions at the earliest time using the trait method
    let surfaces = tlas.detect_collisions(start_position, end_position, new_body_state);

    let mut collision_responses = vec![];

    for surface in &surfaces {
        if let Some(response) = collision::calculate_collision_response(
            h,
            body_derivative,
            new_body_state,
            surface,
            material,
        ) {
            collision_responses.push(response);
        }
    }

    // Combine coplanar collision responses to prevent energy gain
    combine_coplanar_responses(collision_responses, &surfaces)
}

fn combine_coplanar_responses(
    responses: Vec<CollisionResponse>,
    surfaces: &[CollisionSurface],
) -> Vec<CollisionResponse> {
    if responses.is_empty() {
        return responses;
    }

    // An adjustable threshold for considering normals as "coplanar".
    const COPLANAR_THRESHOLD: f32 = 0.999;

    // Creates (index, response) pairs.
    let mut indexed: Vec<(usize, CollisionResponse)> = responses.into_iter().enumerate().collect();

    // Sorts pairs by normal (x, y, z) to group similar normals together.
    indexed.sort_by(|a, b| vec3::lexicographic_cmp(&surfaces[a.0].normal, &surfaces[b.0].normal));

    let mut deduplicated_pairs = vec![];
    let mut finger_index = 0;

    while finger_index < indexed.len() {
        let (left_pair_index, left_pair_response) = &indexed[finger_index];

        let left_pair_normal = surfaces[*left_pair_index].normal;

        deduplicated_pairs.push(*left_pair_response);

        // Skip all subsequent responses with similar normals (coplanar)
        finger_index += 1;

        while finger_index < indexed.len() {
            let (right_pair_index, _right_pair_response) = &indexed[finger_index];

            let right_pair_normal = surfaces[*right_pair_index].normal;

            if left_pair_normal.dot(right_pair_normal) > COPLANAR_THRESHOLD {
                finger_index += 1; // Coplanar; skip this collision.
            } else {
                break; // Found a different normal, start new group.
            }
        }
    }

    deduplicated_pairs
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
        let collision_data = get_triangle_collision_data(tlas, collision);

        let triangle = collision_data.triangle;
        let normal = collision_data.triangle_normal_world;

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
    material: &PhysicsMaterial,
) {
    // Detect all static contacts
    let contacts = tlas.detect_static_contacts(
        body_derivative,
        body_state,
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
