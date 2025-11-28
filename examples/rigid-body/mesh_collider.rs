use cairo::{
    geometry::{
        accelerator::static_triangle_tlas::StaticTriangleTLAS,
        intersect::medial_primitive::intersect_medial_primitive_tlas, primitives::capsule::Capsule,
    },
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::{self, CollisionSurface},
            contact::StaticContact,
            rigid_body::{RigidBodyKind, rigid_body_simulation_state::RigidBodySimulationState},
        },
    },
    vec::vec3::Vec3,
};

use crate::{static_collider::StaticCollider, static_contact_helpers};

impl StaticCollider for StaticTriangleTLAS {
    fn detect_static_contact(
        &self,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        radius: f32,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> Option<StaticContact> {
        let position = new_body_state.position;

        // Create a zero-length capsule at the position (sphere as degenerate capsule)
        let mut capsule = Capsule::new(position, position, radius);

        // Intersect with mesh TLAS
        intersect_medial_primitive_tlas(&mut capsule, self);

        // Check if we hit a triangle
        if let (Some(bvh_instance_index), Some(tri_index)) = (
            capsule.segment.colliding_bvh_index,
            capsule.segment.colliding_primitive,
        ) {
            let bvh_instance = &self.bvh_instances[bvh_instance_index];
            let triangle = &bvh_instance.bvh.tris[tri_index];

            // Get triangle normal in world space
            let triangle_normal = triangle.plane.normal;
            let normal = (triangle_normal * bvh_instance.transform).as_normal();

            // Calculate signed distance to triangle plane
            let signed_distance = triangle.plane.get_signed_distance(&position);

            // Calculate motion tangents
            let body_velocity = new_body_state.velocity();
            let contact_point = position - normal * radius;
            let contact_point_velocity = static_contact_helpers::compute_contact_point_velocity(
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

            static_contact_helpers::get_static_contact(
                derivative,
                new_body_state,
                normal,
                signed_distance,
                radius,
                body_motion_tangent,
                contact_point_motion_tangent,
                material,
                check_forces,
            )
        } else {
            None
        }
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
        if let (Some(bvh_instance_index), Some(tri_index)) = (
            capsule.segment.colliding_bvh_index,
            capsule.segment.colliding_primitive,
        ) {
            let bvh_instance = &self.bvh_instances[bvh_instance_index];
            let triangle = &bvh_instance.bvh.tris[tri_index];

            // Get triangle normal in world space
            let triangle_normal = triangle.plane.normal;
            let normal = (triangle_normal * bvh_instance.transform).as_normal();

            // Calculate collision point
            let collision_point =
                start_position + (end_position - start_position) * capsule.segment.t;

            Some(CollisionSurface {
                normal,
                contact_point: collision_point,
                collision_t: capsule.segment.t,
                minimum_distance: radius,
            })
        } else {
            None
        }
    }

    fn handle_collision(
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

        // Calculate collision response data (if a collision occurred).
        let normal_impulse_data_with_t = calculate_mesh_collision_response(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            self,
            radius,
            material,
        );

        // Applies collision impulses (normal and friction) to new state.
        if let Some((normal_impulse_data, t)) = &normal_impulse_data_with_t {
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

fn calculate_mesh_collision_response(
    h: f32,
    body_derivative: &RigidBodySimulationState,
    current_body_state: &RigidBodySimulationState,
    new_body_state: &mut RigidBodySimulationState,
    tlas: &StaticTriangleTLAS,
    radius: f32,
    material: &PhysicsMaterial,
) -> Option<(
    cairo::physics::simulation::collision::NormalImpulseData,
    f32,
)> {
    let start_position = current_body_state.position;
    let end_position = new_body_state.position;

    // Create capsule from start to end position
    let mut capsule = Capsule::new(start_position, end_position, radius);

    // Intersect capsule with mesh TLAS
    intersect_medial_primitive_tlas(&mut capsule, tlas);

    // Check if we hit a triangle
    if let (Some(bvh_instance_index), Some(tri_index)) = (
        capsule.segment.colliding_bvh_index,
        capsule.segment.colliding_primitive,
    ) {
        let bvh_instance = &tlas.bvh_instances[bvh_instance_index];
        let triangle = &bvh_instance.bvh.tris[tri_index];

        // Get triangle normal in world space
        let triangle_normal = triangle.plane.normal;
        let normal = (triangle_normal * bvh_instance.transform).as_normal();

        // Calculate collision point
        let collision_point = start_position + (end_position - start_position) * capsule.segment.t;

        let surface = CollisionSurface {
            normal,
            contact_point: collision_point,
            collision_t: capsule.segment.t,
            minimum_distance: radius,
        };

        collision::calculate_collision_response(
            h,
            body_derivative,
            new_body_state,
            &surface,
            material,
        )
    } else {
        None
    }
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

    // Check if we hit a triangle
    if let (Some(bvh_instance_index), Some(tri_index)) = (
        capsule.segment.colliding_bvh_index,
        capsule.segment.colliding_primitive,
    ) {
        let bvh_instance = &tlas.bvh_instances[bvh_instance_index];
        let triangle = &bvh_instance.bvh.tris[tri_index];

        // Get triangle normal in world space
        let triangle_normal = triangle.plane.normal;
        let normal = (triangle_normal * bvh_instance.transform).as_normal();

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
    if let Some(contact) = tlas.detect_static_contact(
        body_derivative,
        body_state,
        radius,
        material,
        true, // Post-integration contact detection
    ) {
        // Store the contact for later friction/torque application
        body_state.static_contacts.push(contact).ok();

        // Remove the component of linear momentum pointing into the surface
        let normal = contact.normal;
        body_state.linear_momentum -= normal * body_state.linear_momentum.dot(normal);
    }
}

use crate::state_vector::StateVector;

/// Resolves collisions between multiple rigid bodies and a mesh collider.
/// This function iterates over all rigid bodies, calling the trait method
/// `handle_collision` for each body-collider pair.
pub fn resolve_mesh_collisions(
    mesh_collider: &StaticTriangleTLAS,
    body_count: usize,
    h: f32,
    derivative: &StateVector<RigidBodySimulationState>,
    current_state: &StateVector<RigidBodySimulationState>,
    new_state: &mut StateVector<RigidBodySimulationState>,
    material: &PhysicsMaterial,
) {
    for body_index in 0..body_count {
        let body_derivative = &derivative.0[0];
        let current_body_state = &current_state.0[body_index];
        let new_body_state = &mut new_state.0[body_index];

        mesh_collider.handle_collision(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            material,
        );
    }
}
