use std::f32::consts::PI;

use cairo::{
    color::{self, Color},
    geometry::{
        accelerator::hash_grid::{GridSpaceCoordinate, HashGrid, HashGridInsertionStrategy},
        intersect::{intersect_capsule_plane, intersect_moving_spheres},
        primitives::{plane::Plane, sphere::Sphere},
    },
    matrix::Mat4,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::{
                resolve_rigid_body_collision, resolve_rigid_body_plane_collision,
            },
            contact::{StaticContact, StaticContactKind},
            force::gravity::GRAVITY_RIGID_BODY_FORCE,
            physical_constants::EARTH_GRAVITY_ACCELERATION,
            rigid_body::{
                rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
                RigidBody, RigidBodyKind,
            },
        },
    },
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

use crate::integration::system_dynamics_function;

use crate::{plane_collider::PlaneCollider, state_vector::StateVector};

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
}

impl Simulation {
    pub fn tick(&mut self, h: f32, uptime_seconds: f32) {
        let n = self.rigid_bodies.len();

        let mut state = StateVector::<RigidBodySimulationState>::new(n);

        // Copies current rigid body state into `state`.

        for (i, sphere) in self.rigid_bodies.iter().enumerate() {
            state.0[i] = sphere.into();
        }

        let mut derivative = system_dynamics_function(&state, &self.forces, uptime_seconds);

        // Performs basic forward Euler integration over position and velocity.

        let mut new_state = state.clone() + derivative.clone() * h;

        for (s, d) in new_state.0.iter_mut().zip(derivative.0.iter_mut()) {
            for contact in &s.static_contacts {
                let gravity_projected_onto_contact_normal =
                    contact.normal * EARTH_GRAVITY_ACCELERATION.dot(contact.normal);

                d.linear_momentum += gravity_projected_onto_contact_normal;
                s.linear_momentum += gravity_projected_onto_contact_normal * h;
            }
        }

        for sphere in self.rigid_bodies.iter_mut() {
            sphere.collision_response.take();
        }

        // Detects and resolves collisions with static colliders.

        self.check_static_collisions(
            h,
            &derivative,
            &state,
            &mut new_state,
            &RIGID_BODY_STATIC_PLANE_MATERIAL,
        );

        // Detects and resolves collisions with other (nearby) rigid bodies.

        self.rebuild_hash_grid(&new_state);

        self.check_rigid_bodies_collisions(&state, &mut new_state, &RIGID_BODY_RIGID_BODY_MATERIAL);

        // Copies new state back to rigid bodies.

        for (i, sphere) in self.rigid_bodies.iter_mut().enumerate() {
            sphere.apply_simulation_state(&new_state.0[i]);
        }
    }

    fn check_static_collisions(
        &mut self,
        h: f32,
        derivative: &StateVector<RigidBodySimulationState>,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
        material: &PhysicsMaterial,
    ) {
        for i in 0..self.rigid_bodies.len() {
            let body = &mut self.rigid_bodies[i];

            let current_body_state = &current_state.0[i];
            let new_body_state = &mut new_state.0[i];

            // Resets resting or sliding contact for this tick.

            new_body_state.static_contacts.clear();

            let start_position = current_body_state.position;
            let start_velocity = current_body_state.velocity();

            let end_position = new_body_state.position;

            let end_linear_velocity = new_body_state.velocity();
            let end_angular_velocity = new_body_state.angular_velocity();

            let minimum_distance_to_plane = match body.kind {
                RigidBodyKind::Sphere(radius) => radius,
                _ => panic!(),
            };

            for collider in &self.static_plane_colliders {
                let normal = collider.plane.normal;

                let body_speed_along_normal = end_linear_velocity.dot(normal);

                if body_speed_along_normal > f32::EPSILON {
                    // The sphere is moving away from the plane, so no collision could occur.
                    continue;
                }

                let intersection = match body.kind {
                    RigidBodyKind::Sphere(radius) => intersect_capsule_plane(
                        start_position,
                        end_position,
                        radius,
                        &collider.plane,
                    ),
                    _ => panic!("Unsupported rigid body kind!"),
                };

                if let Some((t, contact_point)) = intersection {
                    if t > 1.0 {
                        // Ignores potential (future) intersection.

                        // Checks for any static contact.

                        if let Some(contact) = Self::get_static_contact(
                            &collider.plane,
                            &new_body_state.position,
                            &new_body_state.velocity(),
                            minimum_distance_to_plane,
                        ) {
                            new_body_state.linear_momentum -= collider.plane.normal
                                * new_body_state.linear_momentum.dot(collider.plane.normal);

                            new_body_state.static_contacts.push(contact).unwrap();
                        }

                        continue;
                    }

                    let r = end_position - contact_point;

                    let contact_point_velocity =
                        end_linear_velocity + end_angular_velocity.cross(r);

                    let incoming_contact_point_speed_normal_to_plane =
                        contact_point_velocity.dot(normal);

                    if incoming_contact_point_speed_normal_to_plane > f32::EPSILON {
                        let signed_distance_from_rigid_body_to_plane =
                            collider.plane.get_signed_distance(&end_position);

                        if signed_distance_from_rigid_body_to_plane < minimum_distance_to_plane {
                            new_body_state.position += normal
                                * (minimum_distance_to_plane
                                    - signed_distance_from_rigid_body_to_plane);
                        }

                        // Checks for any static contact.

                        if let Some(contact) = Self::get_static_contact(
                            &collider.plane,
                            &new_body_state.position,
                            &new_body_state.velocity(),
                            minimum_distance_to_plane,
                        ) {
                            new_body_state.linear_momentum -= collider.plane.normal
                                * new_body_state.linear_momentum.dot(collider.plane.normal);

                            new_body_state.static_contacts.push(contact).unwrap();
                        }

                        continue;
                    }

                    let time_before_collision = h * t;
                    let time_after_collision = h - time_before_collision;

                    {
                        let linear_acceleration = derivative.0[i].linear_momentum;

                        let mass = 1.0 / new_body_state.inverse_mass;

                        let accumulated_linear_velocity =
                            linear_acceleration * time_after_collision;

                        new_body_state.linear_momentum -= accumulated_linear_velocity * mass;
                    }

                    let derivative = &derivative.0[i];

                    let collision_response = resolve_rigid_body_plane_collision(
                        derivative,
                        new_body_state,
                        normal,
                        contact_point,
                        contact_point_velocity,
                        r,
                        material,
                    );

                    let position_at_collision =
                        start_position + start_velocity * time_before_collision;

                    let position_after_collision =
                        position_at_collision + new_body_state.velocity() * time_after_collision;

                    new_body_state.position = position_after_collision;

                    let signed_distance_from_rigid_body_to_plane =
                        collider.plane.get_signed_distance(&new_body_state.position);

                    if signed_distance_from_rigid_body_to_plane < minimum_distance_to_plane {
                        new_body_state.position += normal
                            * (minimum_distance_to_plane
                                - signed_distance_from_rigid_body_to_plane);
                    }

                    body.collision_response.replace(collision_response);
                }

                // Checks for any static contact.

                if let Some(contact) = Self::get_static_contact(
                    &collider.plane,
                    &new_body_state.position,
                    &new_body_state.velocity(),
                    minimum_distance_to_plane,
                ) {
                    new_body_state.linear_momentum -= collider.plane.normal
                        * new_body_state.linear_momentum.dot(collider.plane.normal);

                    new_body_state.static_contacts.push(contact).unwrap();
                }
            }
        }
    }

    fn get_static_contact(
        plane: &Plane,
        position: &Vec3,
        velocity: &Vec3,
        minimum_distance_to_plane: f32,
    ) -> Option<StaticContact> {
        static CONTACT_DISTANCE_THRESHOLD: f32 = 0.001;
        static RESTING_VELOCITY_THRESHOLD: f32 = 0.5;

        let signed_distance_to_plane = plane.get_signed_distance(position);

        if signed_distance_to_plane <= minimum_distance_to_plane + CONTACT_DISTANCE_THRESHOLD {
            let point = position - plane.normal * minimum_distance_to_plane;

            let velocity_along_plane_normal = plane.normal * velocity.dot(plane.normal);
            let velocity_along_plane_normal_mag = velocity_along_plane_normal.mag();

            if velocity_along_plane_normal_mag <= RESTING_VELOCITY_THRESHOLD {
                let (normal, tangent, bitangent) = plane.normal.basis();

                let velocity_along_plane_tangent = velocity - velocity_along_plane_normal;
                let velocity_along_plane_tangent_mag = velocity_along_plane_tangent.mag();

                let kind = if velocity_along_plane_tangent_mag <= RESTING_VELOCITY_THRESHOLD {
                    StaticContactKind::Resting
                } else {
                    StaticContactKind::Sliding
                };

                return Some(StaticContact {
                    kind,
                    point,
                    normal,
                    tangent,
                    bitangent,
                });
            }
        }

        None
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
                            // sphere.collision_response.replace(...);
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
                true,
                Some(color::ORANGE),
            );
        }
    }
}

pub fn make_simulation(sampler: &mut RandomSampler<1024>) -> Simulation {
    // Forces.

    let forces: Vec<Box<DynRigidBodyForce>> = vec![
        // Gravity
        Box::new(GRAVITY_RIGID_BODY_FORCE),
    ];

    // Rigid bodies (spheres).

    static SPHERE_ROWS: usize = 8;
    static SPHERE_COLUMNS: usize = 8;

    static MIN_SPHERE_RADIUS: f32 = 0.5;
    static MAX_SPHERE_RADIUS: f32 = 2.5;

    static SPACING: f32 = 4.0;

    static HEIGHT: f32 = 5.0;

    let mut spheres = Vec::with_capacity(SPHERE_ROWS * SPHERE_COLUMNS);

    static GRID_OFFSET: Vec3 = Vec3 {
        x: 0.5,
        y: 0.5,
        z: 0.5,
    };

    let grid_offset = Vec3 {
        x: -(SPHERE_ROWS as f32 / 2.0) + 0.5,
        y: 0.0,
        z: -(SPHERE_COLUMNS as f32 / 2.0) + 0.5,
    } + GRID_OFFSET;

    for x in 0..SPHERE_ROWS {
        for z in 0..SPHERE_COLUMNS {
            let center = (Vec3 {
                x: x as f32,
                y: 0.0,
                z: z as f32,
            } + grid_offset)
                * SPACING
                + Vec3 {
                    y: HEIGHT,
                    ..Default::default()
                };

            let radius = sampler.sample_range_uniform(MIN_SPHERE_RADIUS, MAX_SPHERE_RADIUS);

            let mass = 3.0;

            let mut sphere = RigidBody::new(RigidBodyKind::Sphere(radius), mass, center);

            let velocity = {
                let speed = sampler.sample_range_uniform(0.0, 4.0);
                let direction = sampler.sample_direction_uniform();

                direction * speed
            };

            sphere.color = Color::from(&mut *sampler);

            sphere.linear_momentum = velocity * sphere.mass;

            let rotation_axis = sampler.sample_direction_uniform();

            let rotation_speed = sampler.sample_range_uniform(-30.0, 30.0);

            sphere.angular_momentum = rotation_axis * rotation_speed;

            spheres.push(sphere);
        }
    }

    // Ground (collider) planes.

    let mut static_plane_colliders = vec![];

    let ground_planes = vec![
        (
            Vec3 {
                x: -20.0,
                ..Default::default()
            },
            Quaternion::new(vec3::FORWARD, PI / 12.0),
        ),
        (
            Vec3 {
                x: 20.0,
                ..Default::default()
            },
            Quaternion::new(vec3::FORWARD, -PI / 12.0),
        ),
        (
            Vec3 {
                z: -30.0,
                ..Default::default()
            },
            Quaternion::new(vec3::RIGHT, -PI / 12.0),
        ),
        (
            Vec3 {
                z: 30.0,
                ..Default::default()
            },
            Quaternion::new(vec3::RIGHT, PI / 12.0),
        ),
    ];

    for (point, rotation) in ground_planes.into_iter() {
        let direction = vec3::UP * *rotation.mat();

        let plane = PlaneCollider::new(point, direction);

        static_plane_colliders.push(plane);
    }

    Simulation {
        forces,
        rigid_bodies: spheres,
        static_plane_colliders,
        hash_grid: HashGrid::new(
            HashGridInsertionStrategy::default(),
            MAX_SPHERE_RADIUS * 2.0,
        ),
    }
}
