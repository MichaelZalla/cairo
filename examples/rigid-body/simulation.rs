use std::f32::consts::PI;

use cairo::{
    color,
    geometry::{
        accelerator::{
            hash_grid::{GridSpaceCoordinate, HashGrid},
            static_triangle_tlas::StaticTriangleTLAS,
        },
        intersect::intersect_moving_spheres,
        primitives::sphere::Sphere,
    },
    matrix::Mat4,
    mesh::Mesh,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision::{
                StaticCollider, colliders::plane_collider::PlaneCollider,
                resolve_rigid_body_collision,
            },
            contact::StaticContactList,
            rigid_body::{
                RigidBody, RigidBodyKind,
                rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
            },
            state_vector::StateVector,
        },
    },
    render::Renderer,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec4::Vec4,
};

use crate::{integration::system_dynamics_function, plane_collider::Renderable};

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
        self.hash_grid.rebuild(final_state.0.iter());

        // 9. (Safety net) Apply corrective impulses for any remaining collisions
        self.apply_corrective_impulses(h, &derivative, &state, &mut final_state);

        // 10. Copy final state back to simulation
        self.apply_state_vector(&final_state);
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
                    h,
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

    pub fn render(&self, renderer: &mut SoftwareRenderer, sphere_mesh: Option<&Mesh>) {
        renderer.render_axes(None);

        // Render dynamic rigid bodies (spheres).
        for sphere in &self.rigid_bodies {
            match sphere_mesh {
                Some(mesh) => {
                    // Render rasterized sphere mesh for the rigid body.
                    let transform_with_radius = match sphere.kind {
                        RigidBodyKind::Sphere(radius) => {
                            Mat4::scale_uniform(radius) * *sphere.transform.mat()
                        }
                        _ => *sphere.transform.mat(),
                    };

                    renderer.render_entity(&transform_with_radius, mesh, &None);
                }
                None => {
                    // Renders a wireframe sphere.
                    sphere.render(renderer);
                }
            }
        }

        // Visualize hash grid entries.
        if self.visualize_hash_grid {
            self.hash_grid.render(renderer, Some(color::LIGHT_GRAY));
        }

        // Visualize static plane colliders.
        for collider in &self.static_plane_colliders {
            collider.render(renderer);
        }

        // Render the ground mesh
        renderer.render_entity(&Mat4::identity(), &self.ground_mesh, &None);

        // Visualize the ground mesh BVH.
        if self.visualize_hash_grid
            && let Some(collider) = self.ground_mesh.collider.as_ref()
        {
            renderer.render_bvh(collider, 2);
        }
    }

    fn copy_to_state_vector(&self) -> StateVector<RigidBodySimulationState> {
        StateVector(self.rigid_bodies.iter().map(|rb| rb.into()).collect())
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

        // Clear static contacts from the last simulation tick.
        for body_state in new_state.0.iter_mut() {
            body_state.static_contacts.clear();
        }

        resolve_plane_collisions(
            &self.static_plane_colliders,
            self.rigid_bodies.len(),
            h,
            derivative,
            current_state,
            new_state,
            &RIGID_BODY_STATIC_PLANE_MATERIAL,
        );

        resolve_mesh_collisions(
            &self.ground_tlas,
            self.rigid_bodies.len(),
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
        self.hash_grid.rebuild(predicted_state.0.iter());

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
            // Extrapolate position forward by timestep h
            let predicted_position = body_state.position + body_state.velocity() * h;

            // Create a predicted body state with the extrapolated position
            let mut predicted_body_state = *body_state;

            predicted_body_state.position = predicted_position;

            // Test against each static plane collider
            for collider in &self.static_plane_colliders {
                let contacts = collider.detect_static_contacts(
                    body_state, // Use current state for derivative (unused when check_forces=false)
                    &predicted_body_state,
                    &RIGID_BODY_STATIC_PLANE_MATERIAL,
                    false, // check_forces: false for prediction
                );

                for contact in contacts.into_iter() {
                    predicted_contacts[body_index].push(contact).ok();
                }
            }

            // Test against triangle mesh collider
            let contacts = self.ground_tlas.detect_static_contacts(
                body_state, // Use current state for derivative (unused when check_forces=false)
                &predicted_body_state,
                &RIGID_BODY_STATIC_PLANE_MATERIAL,
                false, // check_forces: false for prediction
            );

            for contact in contacts.into_iter() {
                predicted_contacts[body_index].push(contact).ok();
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

/// Resolves collisions between multiple rigid bodies and plane colliders.
/// This function iterates over all plane colliders and rigid bodies, calling
/// the trait method `handle_collision` for each body-collider pair.
fn resolve_plane_collisions(
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

            collider.handle_collisions(
                h,
                body_derivative,
                current_body_state,
                new_body_state,
                material,
            );
        }
    }
}

/// Resolves collisions between multiple rigid bodies and a mesh collider.
/// This function iterates over all rigid bodies, calling the trait method
/// `handle_collision` for each body-collider pair.
fn resolve_mesh_collisions(
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

        mesh_collider.handle_collisions(
            h,
            body_derivative,
            current_body_state,
            new_body_state,
            material,
        );
    }
}
