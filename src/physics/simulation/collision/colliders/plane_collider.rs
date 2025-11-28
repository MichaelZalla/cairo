use crate::{
    geometry::{intersect::intersect_capsule_plane, primitives::plane::Plane},
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::{self, CollisionSurface, RESTING_SPEED_THRESHOLD, StaticCollider},
            contact::{self, StaticContactKind},
            rigid_body::{RigidBodyKind, rigid_body_simulation_state::RigidBodySimulationState},
        },
    },
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Clone)]
pub struct PlaneCollider {
    pub point: Vec3,
    pub plane: Plane,
}

impl Default for PlaneCollider {
    fn default() -> Self {
        let point = Vec3::default();

        Self {
            point,
            plane: Plane::new(point, vec3::UP),
        }
    }
}

impl PlaneCollider {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        let plane = Plane::new(point, direction);

        Self { point, plane }
    }
}

impl StaticCollider for PlaneCollider {
    fn detect_static_contacts(
        &self,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        radius: f32,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> contact::StaticContactList<6> {
        let normal = self.plane.normal;

        let signed_distance_to_plane = self.plane.get_signed_distance(&new_body_state.position);

        let body_velocity = new_body_state.velocity();

        let contact_point_velocity = contact::compute_contact_point_velocity(
            body_velocity,
            new_body_state.angular_velocity(),
            new_body_state.position - normal * radius,
            new_body_state.position,
        );

        let body_motion_tangent =
            contact::compute_motion_tangent(self.point, body_velocity, &self.plane);

        let contact_point_motion_tangent =
            contact::compute_motion_tangent(self.point, contact_point_velocity, &self.plane);

        let mut contacts = contact::StaticContactList::default();

        if let Some(contact) = contact::get_static_contact(
            derivative,
            new_body_state,
            normal,
            signed_distance_to_plane,
            radius,
            body_motion_tangent,
            contact_point_motion_tangent,
            material,
            check_forces,
        ) {
            contacts.push(contact).ok();
        }

        contacts
    }

    fn detect_collisions(
        &self,
        start_position: Vec3,
        end_position: Vec3,
        radius: f32,
    ) -> Vec<CollisionSurface> {
        let normal = self.plane.normal;

        // Test for capsule-plane intersection
        if let Some(intersection) =
            intersect_capsule_plane(start_position, end_position, radius, &self.plane)
        {
            vec![CollisionSurface {
                normal,
                contact_point: intersection.contact_point,
                collision_t: intersection.t,
                minimum_distance: radius,
            }]
        } else {
            vec![]
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
        let start_position = current_body_state.position;
        let start_linear_velocity = current_body_state.velocity();
        let end_linear_velocity = new_body_state.velocity();

        // Tests whether any elastic or sliding collision may have occurred.
        let normal = self.plane.normal;

        let body_speed_along_normal = end_linear_velocity.dot(normal);

        if body_speed_along_normal > RESTING_SPEED_THRESHOLD {
            // The rigid body is moving away from the plane
            return;
        }

        // Determines a minimum distance that this body must maintain.
        let minimum_distance_to_plane = match new_body_state.kind {
            RigidBodyKind::Sphere(radius) => radius,
            _ => panic!("Unsupported rigid body kind!"),
        };

        // Calculate collision response data (if a collision occurred).
        let normal_impulse_data_with_t = calculate_plane_collision_response(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            self,
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
        correct_plane_penetration(new_body_state, &self.plane, minimum_distance_to_plane);

        // Determines if the resulting body state constitutes a resting or sliding contact.
        handle_static_contact_after_plane_collision(
            self,
            body_derivative,
            new_body_state,
            minimum_distance_to_plane,
            material,
        );
    }
}

fn calculate_plane_collision_response(
    h: f32,
    body_derivative: &RigidBodySimulationState,
    current_body_state: &RigidBodySimulationState,
    new_body_state: &mut RigidBodySimulationState,
    plane_collider: &PlaneCollider,
    material: &PhysicsMaterial,
) -> Option<(collision::NormalImpulseData, f32)> {
    let start_position = current_body_state.position;
    let end_position = new_body_state.position;

    let radius = match new_body_state.kind {
        RigidBodyKind::Sphere(radius) => radius,
        _ => panic!("Unsupported rigid body kind!"),
    };

    // Detect collisions using the trait method
    let surfaces = plane_collider.detect_collisions(start_position, end_position, radius);

    // For plane, there should be at most one collision
    if let Some(surface) = surfaces.first() {
        collision::calculate_collision_response(
            h,
            body_derivative,
            new_body_state,
            surface,
            material,
        )
    } else {
        None
    }
}

fn correct_plane_penetration(
    body_state: &mut RigidBodySimulationState,
    plane: &Plane,
    minimum_distance_to_plane: f32,
) {
    let signed_distance_from_body_to_plane = plane.get_signed_distance(&body_state.position);

    if signed_distance_from_body_to_plane < minimum_distance_to_plane {
        let normal = plane.normal;

        body_state.position +=
            normal * (minimum_distance_to_plane - signed_distance_from_body_to_plane);
    }
}

fn handle_static_contact_after_plane_collision(
    plane_collider: &PlaneCollider,
    body_derivative: &RigidBodySimulationState,
    body_state: &mut RigidBodySimulationState,
    minimum_distance_to_plane: f32,
    material: &PhysicsMaterial,
) {
    let contacts = plane_collider.detect_static_contacts(
        body_derivative,
        body_state,
        minimum_distance_to_plane,
        material,
        true, // Post-integration contact detection
    );

    for contact in contacts.into_iter() {
        // Removes the component of linear momentum pointing into the collider.
        body_state.linear_momentum -= plane_collider.plane.normal
            * body_state.linear_momentum.dot(plane_collider.plane.normal);

        // Allows spinning around the surface normal, but dampens angular
        // momentum that causes the contact point to slide.
        if matches!(contact.kind, StaticContactKind::Resting) {
            let normal = contact.normal;

            // Decomposes angular momentum into components parallel and
            // perpendicular to normal.
            let angular_momentum_along_normal = normal * body_state.angular_momentum.dot(normal);

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
