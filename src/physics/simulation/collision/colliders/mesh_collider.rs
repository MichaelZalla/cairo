use crate::{
    geometry::{
        CollisionKind,
        accelerator::{
            static_triangle_bvh::StaticBVHCollision, static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::medial_primitive::intersect_medial_primitive_tlas,
        primitives::{
            capsule::Capsule,
            triangle::{Triangle, classify_intersection},
        },
    },
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::{self, CollisionResponse, CollisionSurface, StaticCollider},
            contact::{self, StaticContactList},
            rigid_body::{RigidBodyKind, rigid_body_simulation_state::RigidBodySimulationState},
        },
    },
    vec::vec3::{self, Vec3},
};

/// Result of analyzing a BVH collision against a triangle.
struct TriangleCollisionData<'a> {
    /// The triangle that was hit.
    triangle: &'a Triangle,
    /// The collision point in world space.
    collision_point_world: Vec3,
    /// The type of collision (face, edge, or vertex).
    collision_kind: CollisionKind,
}

/// Computes the collision point on a triangle using barycentric coordinates.
fn compute_collision_point(barycentric: Vec3, triangle_vertices: (Vec3, Vec3, Vec3)) -> Vec3 {
    let (v_a, v_b, v_c) = triangle_vertices;

    let u = barycentric.x;
    let v = barycentric.y;
    let w = barycentric.z;

    v_a * u + v_b * v + v_c * w
}

/// Extracts triangle collision data from a BVH collision.
fn get_triangle_collision_data<'a>(
    tlas: &'a StaticTriangleTLAS,
    collision: &StaticBVHCollision,
) -> TriangleCollisionData<'a> {
    let bvh_instance_index = collision.colliding_bvh_index;
    let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

    let triangle_index = collision.colliding_primitive;
    let triangle = &bvh_instance.bvh.tris[triangle_index];

    // Classify the collision type based on barycentric coordinates
    let collision_kind = classify_intersection(collision.barycentric);

    // Get triangle vertices in local space
    let triangle_vertices_local = bvh_instance.bvh.geometry.get_triangle_vertices(triangle);

    // Compute collision point in local space
    let collision_point_local =
        compute_collision_point(collision.barycentric, triangle_vertices_local);

    // Transform collision point to world space
    let collision_point_world = collision_point_local * bvh_instance.transform;

    TriangleCollisionData {
        triangle,
        collision_point_world,
        collision_kind,
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

        let radius = match new_body_state.kind {
            RigidBodyKind::Sphere(r) => r,
            _ => panic!("Unsupported rigid body kind"),
        };

        // Create a capsule extending downward to find nearby triangles within contact distance
        // This ensures the BVH finds triangles even when the sphere is slightly above the surface
        let query_extension = contact::CONTACT_DISTANCE_THRESHOLD;
        let downward_offset = Vec3 {
            x: 0.0,
            y: -query_extension,
            z: 0.0,
        };
        let query_end = position + downward_offset;
        let mut capsule = Capsule::new(position, query_end, radius);

        // Intersect with mesh TLAS to find all collisions
        intersect_medial_primitive_tlas(&mut capsule, self);

        let mut contacts = contact::StaticContactList::default();

        // Process all detected collisions
        for collision in capsule.segment.collisions.iter() {
            let bvh_instance_index = collision.colliding_bvh_index;
            let bvh_instance = &self.bvh_instances[bvh_instance_index];

            let collision_data = get_triangle_collision_data(self, collision);

            let normal = new_body_state.get_triangle_collision_normal(
                collision_data.triangle,
                collision_data.collision_point_world,
                &collision_data.collision_kind,
                bvh_instance,
            );

            // Calculate signed distance to triangle plane
            let signed_distance = collision_data.triangle.plane.get_signed_distance(&position);

            // Calculate motion tangents
            let body_velocity = new_body_state.velocity();

            let contact_point = position - normal * radius;

            let contact_point_velocity = contact::compute_contact_point_velocity(
                body_velocity,
                new_body_state.angular_velocity(),
                contact_point,
                position,
            );

            // Use triangle plane for motion tangent calculation (consistent with PlaneCollider)
            let point_on_triangle = collision_data.collision_point_world;
            let triangle_plane = &collision_data.triangle.plane;

            let body_motion_tangent =
                contact::compute_motion_tangent(point_on_triangle, body_velocity, triangle_plane);

            let contact_point_motion_tangent = contact::compute_motion_tangent(
                point_on_triangle,
                contact_point_velocity,
                triangle_plane,
            );

            let contact_result = contact::get_static_contact(
                derivative,
                new_body_state,
                normal,
                signed_distance,
                radius,
                body_motion_tangent,
                contact_point_motion_tangent,
                material,
                check_forces,
            );

            if let Some(contact) = contact_result {
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
        let radius = match body_state.kind {
            RigidBodyKind::Sphere(r) => r,
            _ => panic!("Unsupported rigid body kind"),
        };

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
                    // Calculate body position at collision time
                    let body_position_at_collision =
                        start_position + (end_position - start_position) * collision.t;

                    let bvh_instance_index = collision.colliding_bvh_index;
                    let bvh_instance = &self.bvh_instances[bvh_instance_index];

                    let collision_data = get_triangle_collision_data(self, collision);

                    // Create a minimal rigid body state for computing the collision normal
                    let temp_body_state = RigidBodySimulationState {
                        kind: RigidBodyKind::Sphere(radius),
                        position: body_position_at_collision,
                        ..Default::default()
                    };

                    let normal = temp_body_state.get_triangle_collision_normal(
                        collision_data.triangle,
                        collision_data.collision_point_world,
                        &collision_data.collision_kind,
                        bvh_instance,
                    );

                    // let triangle_surface_point = collision_data.collision_point_world;
                    let sphere_surface_point = body_position_at_collision - normal * radius;

                    // BUG FIX: Use sphere surface point (like PlaneCollider), not triangle surface point
                    let contact_point = sphere_surface_point;

                    surfaces.push(CollisionSurface {
                        normal,
                        contact_point,
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
        // Get sphere radius for penetration correction
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
            material,
        );

        if !collision_responses.is_empty() {
            println!(
                "[MESH COLLISION] Detected {} collision responses",
                collision_responses.len()
            );
        }

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

        // Debug: Check velocity right before contact detection
        // let vel_before_contact_detect = new_body_state.velocity();
        // println!("[BEFORE_CONTACT_DETECT] vel=({:.3}, {:.3}, {:.3})",
        //          vel_before_contact_detect.x, vel_before_contact_detect.y, vel_before_contact_detect.z);

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

    // Capture velocity before rewinding for threshold checks
    let end_velocity_before_rewind = new_body_state.velocity();

    // BUG FIX #3: Remove post-collision momentum ONCE before processing all surfaces
    // (not once per surface, which would cause multiple rewinds)
    if !surfaces.is_empty() {
        let earliest_collision_t = surfaces
            .iter()
            .map(|s| s.collision_t)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        collision::remove_post_collision_momentum(
            h,
            earliest_collision_t,
            body_derivative,
            new_body_state,
        );
    }

    let mut collision_responses = vec![];

    for surface in &surfaces {
        // Calculate response WITHOUT rewinding (already rewound above)
        // Pass the pre-rewind velocity for threshold checking
        if let Some(response) = collision::calculate_collision_response_no_rewind(
            new_body_state,
            surface,
            material,
            end_velocity_before_rewind,
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

// fn dampen_velocity_near_surface(
//     body_state: &mut RigidBodySimulationState,
//     tlas: &StaticTriangleTLAS,
//     radius: f32,
// ) {
//     let position = body_state.position;

//     // Create a zero-length capsule at the position
//     let mut capsule = Capsule::new(position, position, radius);

//     // Intersect with mesh TLAS
//     intersect_medial_primitive_tlas(&mut capsule, tlas);

//     // For each nearby triangle, dampen velocity pointing into it
//     for collision in capsule.segment.collisions.iter() {
//         let bvh_instance_index = collision.colliding_bvh_index;
//         let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

//         let collision_data = get_triangle_collision_data(tlas, collision);

//         let normal = body_state.get_triangle_collision_normal(
//             collision_data.triangle,
//             collision_data.collision_point_world,
//             collision_data.collision_kind,
//             bvh_instance,
//         );

//         // Calculate signed distance from body to triangle plane
//         let signed_distance = collision_data
//             .triangle
//             .plane
//             .get_signed_distance(&body_state.position);

//         // If very close to the surface (within radius + small tolerance)
//         if signed_distance <= radius + 0.1 {
//             // Remove the component of linear momentum pointing into the surface
//             let momentum_into_surface = body_state.linear_momentum.dot(normal);
//             if momentum_into_surface < 0.0 {
//                 body_state.linear_momentum -= normal * momentum_into_surface;
//             }
//         }
//     }
// }

fn correct_mesh_penetration(
    body_state: &mut RigidBodySimulationState,
    tlas: &StaticTriangleTLAS,
    radius: f32,
) {
    // TODO: Potential issue with using stale collision data (collision_point_world)
    // after position updates when correcting multiple penetrations in a loop.
    // This could lead to incorrect normals and overshooting corrections.

    let position = body_state.position;

    // Create a zero-length capsule at the position
    let mut capsule = Capsule::new(position, position, radius);

    // Intersect with mesh TLAS
    intersect_medial_primitive_tlas(&mut capsule, tlas);

    // Process all detected collisions to correct penetration
    for collision in capsule.segment.collisions.iter() {
        let bvh_instance_index = collision.colliding_bvh_index;
        let bvh_instance = &tlas.bvh_instances[bvh_instance_index];

        let collision_data = get_triangle_collision_data(tlas, collision);

        let normal = body_state.get_triangle_collision_normal(
            collision_data.triangle,
            collision_data.collision_point_world,
            &collision_data.collision_kind,
            bvh_instance,
        );

        // Calculate signed distance from body to triangle plane
        let signed_distance_from_body_to_plane = collision_data
            .triangle
            .plane
            .get_signed_distance(&body_state.position);

        // If penetrating, push the body out
        if signed_distance_from_body_to_plane < radius {
            let penetration_depth = radius - signed_distance_from_body_to_plane;
            let correction = normal * penetration_depth;

            body_state.position += correction;
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

    // Store all contacts for later friction/torque application
    for contact in contacts.iter() {
        body_state.static_contacts.push(*contact).ok();
    }

    // For mesh colliders, only remove momentum to prevent micro-bouncing,
    // not to stop rolling/sliding motion
    let mut deepest_penetration = f32::MIN;

    for contact in contacts.iter() {
        if contact.penetration_depth > deepest_penetration {
            deepest_penetration = contact.penetration_depth;
        }
    }

    // Only apply momentum removal if:
    // 1. Actually penetrating (not just nearby)
    // 2. Velocity is very small (micro-bouncing, not rolling/sliding)
    const PENETRATION_THRESHOLD: f32 = 0.0;
    const MICRO_BOUNCE_VELOCITY_THRESHOLD: f32 = 0.5; // m/s

    if deepest_penetration > PENETRATION_THRESHOLD {
        let velocity = body_state.velocity();
        let speed = velocity.mag();

        // Only remove vertical momentum if moving very slowly (micro-bouncing)
        // Don't interfere with rolling/sliding motion
        if speed < MICRO_BOUNCE_VELOCITY_THRESHOLD && velocity.y < 0.0 {
            body_state.linear_momentum.y = 0.0;
        }
    }
}
