use std::f32::consts::PI;

use cairo::{
    color,
    geometry::{
        accelerator::hash_grid::{GridSpaceCoordinate, HashGrid},
        intersect::{intersect_capsule_plane, intersect_moving_spheres},
        primitives::sphere::Sphere,
    },
    matrix::Mat4,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::{
                get_rigid_body_plane_friction_impulse, get_rigid_body_plane_normal_impulse,
                resolve_rigid_body_collision,
            },
            contact::{StaticContact, StaticContactKind},
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
    vec::{vec3, vec3::Vec3, vec4::Vec4},
};

use crate::integration::system_dynamics_function;

use crate::{plane_collider::PlaneCollider, state_vector::StateVector};

static CONTACT_DISTANCE_THRESHOLD: f32 = 0.005;

static RESTING_SPEED_THRESHOLD: f32 = 0.05;

static RIGID_BODY_STATIC_PLANE_MATERIAL: PhysicsMaterial = PhysicsMaterial {
    static_friction: PI / 4.0,
    dynamic_friction: 0.6,
    restitution: 0.4,
};

static RIGID_BODY_RIGID_BODY_MATERIAL: PhysicsMaterial = PhysicsMaterial {
    static_friction: PI / 4.0,
    dynamic_friction: 0.6,
    restitution: 0.4,
};

pub struct Simulation {
    pub forces: Vec<Box<DynRigidBodyForce>>,
    pub rigid_bodies: Vec<RigidBody>,
    pub static_plane_colliders: Vec<PlaneCollider>,
    pub hash_grid: HashGrid,
    pub visualize_hash_grid: bool,
}

impl Simulation {
    pub fn tick(&mut self, h: f32, uptime_seconds: f32) {
        let state = self.copy_to_state_vector();

        let derivative = system_dynamics_function(&state, &self.forces, h, uptime_seconds);

        // Semi-implicit Euler integration: update momenta from forces,
        // then update position and orientation from the new momenta.

        let mut new_state = state.clone();

        for (body, derivative) in new_state.0.iter_mut().zip(derivative.0.iter()) {
            // Update linear momentum from forces.
            body.linear_momentum += derivative.linear_momentum * h;

            // Update angular momentum from torques.
            body.angular_momentum += derivative.angular_momentum * h;

            // Update position from new linear momentum (semi-implicit step).
            body.position += body.linear_momentum * body.inverse_mass * h;

            // Compute angular velocity from new angular momentum.
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
        }

        self.clear_collision_debug_info();

        // Detects and resolves collisions with static colliders.

        self.handle_static_collisions(
            h,
            &derivative,
            &state,
            &mut new_state,
            &RIGID_BODY_STATIC_PLANE_MATERIAL,
        );

        // Detects and resolves collisions with other (nearby) rigid bodies.

        self.rebuild_hash_grid(&new_state);

        self.check_rigid_bodies_collisions(&state, &mut new_state, &RIGID_BODY_RIGID_BODY_MATERIAL);

        self.apply_state_vector(&new_state);
    }

    fn handle_static_collisions(
        &mut self,
        h: f32,
        derivative: &StateVector<RigidBodySimulationState>,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
        material: &PhysicsMaterial,
    ) {
        for (collider_index, collider) in self.static_plane_colliders.iter().enumerate() {
            for body_index in 0..self.rigid_bodies.len() {
                let body = &mut self.rigid_bodies[body_index];

                let body_derivative = &derivative.0[0];
                let current_body_state = &current_state.0[body_index];
                let new_body_state = &mut new_state.0[body_index];

                if collider_index == 0 {
                    // Clears any resting or sliding contacts from the last simulation tick.

                    new_body_state.static_contacts.clear();
                }

                Self::handle_static_collision(
                    h,
                    body,
                    body_derivative,
                    current_body_state,
                    new_body_state,
                    collider,
                    material,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_static_collision(
        h: f32,
        body: &mut RigidBody,
        body_derivative: &RigidBodySimulationState,
        current_body_state: &RigidBodySimulationState,
        new_body_state: &mut RigidBodySimulationState,
        collider: &PlaneCollider,
        material: &PhysicsMaterial,
    ) {
        let start_position = current_body_state.position;
        let start_linear_velocity = current_body_state.velocity();

        let end_position = new_body_state.position;
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

        let minimum_distance_to_plane = match body.kind {
            RigidBodyKind::Sphere(radius) => radius,
            _ => panic!(),
        };

        // Tests for a body-collider intersection, according to the kind of
        // rigid body.

        let intersection = match body.kind {
            RigidBodyKind::Sphere(radius) => {
                intersect_capsule_plane(start_position, end_position, radius, &collider.plane)
            }
            _ => panic!("Unsupported rigid body kind!"),
        };

        let normal_impulse_data_with_t = match intersection {
            Some((t, contact_point)) => {
                if t > 1.0 {
                    // A t-value greater than 1 indicates a future collision, which
                    // we'll ignore for this tick.

                    None
                } else {
                    // Computes the moment arm from the rigid body's
                    // center-of-mass to the colliding contact point.

                    let r = end_position - contact_point;

                    // Computes the linear velocity of the contact point in our
                    // absolute (world-space) frame of reference.

                    let contact_point_velocity =
                        end_linear_velocity + new_body_state.angular_velocity().cross(r);

                    // Calculates the amount that this velocity projects onto
                    // the collider's normal; note that a positive quantity
                    // indicates a velocity that moves out and away from the plane.

                    let incoming_contact_point_speed_normal_to_plane =
                        contact_point_velocity.dot(normal);

                    // Tests that the resulting velocity projects into the
                    // plane, i.e., the point is moving into the plane.

                    if incoming_contact_point_speed_normal_to_plane < -RESTING_SPEED_THRESHOLD {
                        // At this point, we know that the rigid body intersects
                        // the collider, and that its point-of-contact is moving
                        // deeper into the collider. These are sufficient conditions
                        // for applying a normal impulse (i.e., a collision response).

                        // Removes any linear momentum gained while colliding.

                        {
                            let time_before_collision = h * t;

                            let time_after_collision = h - time_before_collision;

                            let linear_acceleration = body_derivative.linear_momentum;

                            let mass = 1.0 / new_body_state.inverse_mass;

                            let accumulated_linear_velocity =
                                linear_acceleration * time_after_collision;

                            new_body_state.linear_momentum -= accumulated_linear_velocity * mass;
                        }

                        // Calculates the magnitude of the normal impulse for
                        // this collision, based on the contact point's
                        // velocity, the plane normal, and the physics material.

                        Some((
                            get_rigid_body_plane_normal_impulse(
                                new_body_state,
                                normal,
                                contact_point,
                                contact_point_velocity,
                                r,
                                material,
                            ),
                            t,
                        ))
                    } else {
                        None
                    }
                }
            }
            None => None,
        };

        // Applies normal impulse to new state.

        if let Some((normal_impulse_data, t)) = &normal_impulse_data_with_t {
            let normal = normal_impulse_data.normal;
            let normal_impulse_magnitude = normal_impulse_data.magnitude;

            let r = normal_impulse_data.r;

            // Applies normal impulse to linear momentum.

            new_body_state.linear_momentum += normal * normal_impulse_magnitude;

            // Applies normal impulse to angular momentum.

            let rotation_axis = r.cross(normal);

            new_body_state.angular_momentum += rotation_axis * normal_impulse_magnitude;

            // Determines the body's new position, using both velocities.

            {
                let time_before_collision = h * t;
                let time_after_collision = h - time_before_collision;

                new_body_state.position = {
                    let position_at_collision =
                        start_position + start_linear_velocity * time_before_collision;

                    position_at_collision + new_body_state.velocity() * time_after_collision
                };
            }

            // Applies tangent impulse (if any) to new state.

            if let Some(tangent_impulse_data) = get_rigid_body_plane_friction_impulse(
                1.0 / new_body_state.inverse_mass,
                body_derivative,
                normal,
                normal_impulse_data.contact_point_velocity,
                normal_impulse_magnitude,
                material,
            ) {
                let tangent = tangent_impulse_data.tangent;
                let magnitude = tangent_impulse_data.magnitude;

                let tangent_impulse = tangent * magnitude;

                // Applies tangent impulse to linear momentum.

                new_body_state.linear_momentum += tangent_impulse;

                // Applies tangent impulse to angular momentum.

                new_body_state.angular_momentum += r.cross(tangent) * magnitude;
            }
        }

        // If necessary, fixes up the body's position to maintain its minimum
        // distance from the collider.

        let signed_distance_from_body_to_plane =
            collider.plane.get_signed_distance(&new_body_state.position);

        if signed_distance_from_body_to_plane < minimum_distance_to_plane {
            new_body_state.position +=
                normal * (minimum_distance_to_plane - signed_distance_from_body_to_plane);
        }

        // Determines if the resulting body state constitutes a resting or
        // a sliding contact.

        if let Some(contact) = Self::get_static_contact(
            collider,
            body_derivative,
            new_body_state,
            minimum_distance_to_plane,
            material,
        ) {
            // Removes the component of linear momentum pointing into the collider.

            new_body_state.linear_momentum -=
                collider.plane.normal * new_body_state.linear_momentum.dot(collider.plane.normal);

            // Records the contact.

            new_body_state.static_contacts.push(contact).unwrap();
        }
    }

    fn get_static_contact(
        collider: &PlaneCollider,
        derivative: &RigidBodySimulationState,
        new_body_state: &RigidBodySimulationState,
        minimum_distance_to_plane: f32,
        material: &PhysicsMaterial,
    ) -> Option<StaticContact> {
        let normal = collider.plane.normal;

        let signed_distance_to_plane = collider.plane.get_signed_distance(&new_body_state.position);

        // 1. Requires that the body be very close to the collider
        //    (within some threshold).

        if signed_distance_to_plane > minimum_distance_to_plane + CONTACT_DISTANCE_THRESHOLD {
            return None;
        }

        // 2. Requires that the contact point's speed along the normal be very small
        //    (within some threshold).

        let body_velocity = new_body_state.velocity();

        let contact_point = new_body_state.position - normal * minimum_distance_to_plane;

        let contact_point_velocity = body_velocity
            - new_body_state
                .angular_velocity()
                .cross(contact_point - new_body_state.position);

        let contact_point_speed_along_normal = contact_point_velocity.dot(normal);

        if contact_point_speed_along_normal.abs() > RESTING_SPEED_THRESHOLD {
            return None;
        }

        // 3. Requires that the forces acting on the body aren't pulling the
        //    body away from the collider.

        let magnitude_of_acceleration_along_normal = derivative.linear_momentum.dot(normal);

        // @NOTE For a rigid body shape that is more complex than a sphere, the
        // body may be moving away from the plane, while the point of contact is
        // moving into it!

        if magnitude_of_acceleration_along_normal > f32::EPSILON {
            return None;
        }

        // At this point, we can be certain that the object is resting, rolling
        // along the plane, or sliding along the plane.

        let body_motion_tangent = {
            let v = collider.point + body_velocity;

            let d = collider.plane.get_signed_distance(&v);

            let body_velocity_projected_onto_plane = v - collider.plane.normal * d;

            if body_velocity_projected_onto_plane.is_zero() {
                None
            } else {
                Some(body_velocity_projected_onto_plane.as_normal())
            }
        };

        let contact_point_motion_tangent = {
            let v = collider.point + contact_point_velocity;

            let d = collider.plane.get_signed_distance(&v);

            let contact_point_velocity_projected_onto_plane = v - collider.plane.normal * d;

            if contact_point_velocity_projected_onto_plane.is_zero() {
                None
            } else {
                Some(contact_point_velocity_projected_onto_plane.as_normal())
            }
        };

        let kind = {
            // Determines whether or not the external forces in the tangential
            // direction are large enough to overcome static friction.

            let body_speed_along_tangent = if let Some(tangent) = body_motion_tangent {
                body_velocity.dot(tangent)
            } else {
                0.0
            };

            let contact_point_speed_along_tangent =
                if let Some(tangent) = contact_point_motion_tangent {
                    contact_point_velocity.dot(tangent)
                } else {
                    0.0
                };

            if body_speed_along_tangent < RESTING_SPEED_THRESHOLD
                && contact_point_speed_along_tangent < RESTING_SPEED_THRESHOLD
            {
                StaticContactKind::Resting
            } else if contact_point_speed_along_tangent < RESTING_SPEED_THRESHOLD {
                StaticContactKind::Rolling(0.5)
            } else {
                StaticContactKind::Sliding
            }
        };

        let (tangent, bitangent) = match contact_point_motion_tangent {
            Some(tangent) => (tangent, tangent.cross(normal)),
            None => {
                let (_, tangent, bitangent) = normal.basis();

                (tangent, bitangent)
            }
        };

        Some(StaticContact {
            kind,
            point: contact_point,
            normal,
            tangent,
            bitangent,
            contact_point_velocity,
            material: *material,
        })
    }

    fn check_rigid_bodies_collisions(
        &mut self,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
        material: &PhysicsMaterial,
    ) {
        for current_body_index in 0..self.rigid_bodies.len() {
            let body = &new_state.0[current_body_index];

            let (strategy, scale) = (self.hash_grid.strategy, self.hash_grid.scale);

            let body_coordinate = GridSpaceCoordinate::from((body, strategy, scale));

            static NEIGHBORS_UP_FORWARD_RIGHT: [(isize, isize, isize); 14] = [
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

            let neighbors = &NEIGHBORS_UP_FORWARD_RIGHT;

            for (x_offset, y_offset, z_offset) in neighbors.iter() {
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
            Some((_t, contact_point)) => {
                // Compute and apply the collision response.

                let mut s1_state_cloned = new_state.0[a];
                let mut s2_state_cloned = new_state.0[b];

                resolve_rigid_body_collision(
                    &mut s1_state_cloned,
                    &mut s2_state_cloned,
                    contact_point,
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

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        // Render dynamic rigid bodies (spheres).

        for sphere in &self.rigid_bodies {
            sphere.render(renderer);
        }

        // Visualize hash grid entries.

        if self.visualize_hash_grid {
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

        for collider in &self.static_plane_colliders {
            // Visualize static plane colliders.

            let mut right = collider.plane.normal.cross(vec3::UP);

            if right.mag() < f32::EPSILON {
                right = collider.plane.normal.cross(vec3::FORWARD);
            }

            right = right.as_normal();

            let up = collider.plane.normal.cross(-right);

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

            renderer.render_line(
                collider.point,
                collider.point + collider.plane.normal * 5.0,
                color::GREEN,
            );
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
}
