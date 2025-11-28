use cairo::{
    color,
    geometry::{intersect::intersect_capsule_plane, primitives::plane::Plane},
    matrix::Mat4,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::CollisionSurface,
            contact::{StaticContact, StaticContactKind},
            rigid_body::{RigidBodyKind, rigid_body_simulation_state::RigidBodySimulationState},
        },
    },
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    vec::vec3::{self, Vec3},
};

use crate::{
    collision_helpers,
    state_vector::StateVector,
    static_collider::StaticCollider,
    static_contact_helpers::{self, RESTING_SPEED_THRESHOLD},
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

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        // Calculate the coordinate frame for the plane
        let (_forward, right, up) = self.plane.normal.basis();

        // Render the plane as an orange square
        let scale = Mat4::scale_uniform(20.0);
        let translate = Mat4::translation(self.point);
        let rotate = Mat4::tbn(right, up, self.plane.normal);

        let transform = scale * rotate * translate;

        renderer.render_empty(
            &transform,
            EmptyDisplayKind::Square,
            false,
            Some(color::ORANGE),
        );

        // Render the normal vector as a green line
        renderer.render_line(
            self.point,
            self.point + self.plane.normal * 5.0,
            color::GREEN,
        );
    }
}

impl StaticCollider for PlaneCollider {
    fn detect_static_contact(
        &self,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        radius: f32,
        material: &PhysicsMaterial,
        check_forces: bool,
    ) -> Option<StaticContact> {
        let normal = self.plane.normal;

        let signed_distance_to_plane = self.plane.get_signed_distance(&new_body_state.position);

        let body_velocity = new_body_state.velocity();

        let contact_point_velocity = static_contact_helpers::compute_contact_point_velocity(
            body_velocity,
            new_body_state.angular_velocity(),
            new_body_state.position - normal * radius,
            new_body_state.position,
        );

        let body_motion_tangent =
            static_contact_helpers::compute_motion_tangent(self.point, body_velocity, &self.plane);

        let contact_point_motion_tangent = static_contact_helpers::compute_motion_tangent(
            self.point,
            contact_point_velocity,
            &self.plane,
        );

        static_contact_helpers::get_static_contact(
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

    fn detect_collision(
        &self,
        start_position: Vec3,
        end_position: Vec3,
        radius: f32,
    ) -> Option<CollisionSurface> {
        let normal = self.plane.normal;

        // Test for capsule-plane intersection
        let intersection =
            intersect_capsule_plane(start_position, end_position, radius, &self.plane)?;

        Some(CollisionSurface {
            normal,
            contact_point: intersection.contact_point,
            collision_t: intersection.t,
            minimum_distance: radius,
        })
    }

    fn handle_collision(
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
        let normal_impulse_data_with_t = self.calculate_collision_response_internal(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            minimum_distance_to_plane,
            material,
        );

        // Applies collision impulses (normal and friction) to new state.
        if let Some((normal_impulse_data, t)) = &normal_impulse_data_with_t {
            collision_helpers::apply_collision_impulses(
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
        self.correct_penetration_internal(new_body_state, minimum_distance_to_plane);

        // Determines if the resulting body state constitutes a resting or sliding contact.
        self.handle_static_contact_after_collision_internal(
            body_derivative,
            new_body_state,
            minimum_distance_to_plane,
            material,
        );
    }
}

impl PlaneCollider {
    fn calculate_collision_response_internal(
        &self,
        h: f32,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        minimum_distance_to_plane: f32,
        material: &PhysicsMaterial,
    ) -> Option<(
        cairo::physics::simulation::collision::NormalImpulseData,
        f32,
    )> {
        let start_position = current_body_state.position;
        let end_position = new_body_state.position;

        let normal = self.plane.normal;

        // Tests for a body-collider intersection
        let intersection = match new_body_state.kind {
            RigidBodyKind::Sphere(radius) => {
                intersect_capsule_plane(start_position, end_position, radius, &self.plane)
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

                collision_helpers::calculate_collision_response(
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

    fn correct_penetration_internal(
        &self,
        body_state: &mut RigidBodySimulationState,
        minimum_distance_to_plane: f32,
    ) {
        let signed_distance_from_body_to_plane =
            self.plane.get_signed_distance(&body_state.position);

        if signed_distance_from_body_to_plane < minimum_distance_to_plane {
            let normal = self.plane.normal;

            body_state.position +=
                normal * (minimum_distance_to_plane - signed_distance_from_body_to_plane);
        }
    }

    fn handle_static_contact_after_collision_internal(
        &self,
        body_derivative: &RigidBodySimulationState,
        body_state: &mut RigidBodySimulationState,
        minimum_distance_to_plane: f32,
        material: &PhysicsMaterial,
    ) {
        if let Some(contact) = self.detect_static_contact(
            body_derivative,
            body_state,
            minimum_distance_to_plane,
            material,
            true, // Post-integration contact detection
        ) {
            // Removes the component of linear momentum pointing into the collider.
            body_state.linear_momentum -=
                self.plane.normal * body_state.linear_momentum.dot(self.plane.normal);

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
}

/// Resolves collisions between multiple rigid bodies and plane colliders.
/// This function iterates over all plane colliders and rigid bodies, calling
/// the trait method `handle_collision` for each body-collider pair.
pub fn resolve_plane_collisions(
    plane_colliders: &[PlaneCollider],
    body_count: usize,
    h: f32,
    derivative: &StateVector<RigidBodySimulationState>,
    current_state: &StateVector<RigidBodySimulationState>,
    new_state: &mut StateVector<RigidBodySimulationState>,
    material: &PhysicsMaterial,
) {
    for collider in plane_colliders.iter() {
        for body_index in 0..body_count {
            let body_derivative = &derivative.0[0];
            let current_body_state = &current_state.0[body_index];
            let new_body_state = &mut new_state.0[body_index];

            collider.handle_collision(
                h,
                body_derivative,
                current_body_state,
                new_body_state,
                material,
            );
        }
    }
}
