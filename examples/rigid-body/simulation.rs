use std::f32::consts::PI;

use cairo::{
    color::{self, Color},
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
                resolve_rigid_body_collision, resolve_rigid_body_plane_collision,
            },
            force::gravity::GRAVITY_RIGID_BODY_FORCE,
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

static SPHERE_RADIUS: f32 = 0.5;
static SPHERE_MASS: f32 = 1.0;

static PHYSICS_MATERIAL: PhysicsMaterial = PhysicsMaterial {
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

        let derivative = system_dynamics_function(&state, &self.forces, uptime_seconds);

        // Performs basic forward Euler integration over position and velocity.

        let mut new_state = state.clone() + derivative.clone() * h;

        for sphere in self.rigid_bodies.iter_mut() {
            sphere.collision_response.take();
        }

        // Detects and resolves collisions with static colliders.

        self.check_static_collisions(h, &derivative, &state, &mut new_state);

        // Detects and resolves collisions with other (nearby) rigid bodies.

        self.rebuild_hash_grid(&new_state);

        self.check_rigid_bodies_collisions(&state, &mut new_state);

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
    ) {
        for i in 0..self.rigid_bodies.len() {
            let sphere = &mut self.rigid_bodies[i];

            let radius = match sphere.kind {
                RigidBodyKind::Sphere(radius) => radius,
                _ => panic!(),
            };

            let current_body_state = &current_state.0[i];
            let new_body_state = &mut new_state.0[i];

            let start_position = current_body_state.position;
            let start_velocity = current_body_state.velocity();

            let end_position = new_body_state.position;

            let end_linear_velocity = new_body_state.velocity();
            let end_angular_velocity = new_body_state.angular_velocity();

            for collider in &self.static_plane_colliders {
                let normal = collider.plane.normal;

                let body_speed_along_normal = end_linear_velocity.dot(normal);

                if body_speed_along_normal > f32::EPSILON {
                    // The sphere is moving away from the plane, so no collision could occur.
                    continue;
                }

                if let Some((t, contact_point)) =
                    intersect_capsule_plane(start_position, end_position, radius, &collider.plane)
                {
                    if t > 1.0 {
                        // Ignores potential (future) intersection.
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

                        if signed_distance_from_rigid_body_to_plane <= radius {
                            new_body_state.position +=
                                normal * (radius - signed_distance_from_rigid_body_to_plane + 0.01);
                        }

                        continue;
                    }

                    let time_before_collision = h * t;
                    let time_after_collision = h - time_before_collision;

                    {
                        let linear_acceleration = derivative.0[i].linear_momentum;

                        let mass = 1.0 / new_body_state.inverse_mass;

                        let accumulated_linear_velocity =
                            linear_acceleration * 2.0 * time_after_collision;

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
                        &PHYSICS_MATERIAL,
                    );

                    let position_at_collision =
                        start_position + start_velocity * time_before_collision;

                    let position_after_collision = position_at_collision
                        + new_body_state.velocity() * time_after_collision
                        + normal * 0.01;

                    new_body_state.position = position_after_collision;

                    let signed_distance_from_rigid_body_to_plane =
                        collider.plane.get_signed_distance(&new_body_state.position);

                    if signed_distance_from_rigid_body_to_plane <= radius {
                        new_body_state.position +=
                            normal * (radius - signed_distance_from_rigid_body_to_plane + 0.01);
                    }

                    sphere.collision_response.replace(collision_response);
                }
            }
        }
    }

    fn check_rigid_bodies_collisions(
        &mut self,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
    ) {
        for (current_sphere_index, _sphere) in self.rigid_bodies.iter_mut().enumerate() {
            let sphere_state = &new_state.0[current_sphere_index];

            let current_grid_coord =
                GridSpaceCoordinate::from((sphere_state, self.hash_grid.scale));

            for x_offset in -1..=1 {
                for y_offset in -1..=1 {
                    for z_offset in -1..=1 {
                        if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                            // Checks current cell.

                            let cell = self.hash_grid.map.get(&current_grid_coord).unwrap();

                            for sphere_index in cell {
                                if *sphere_index != current_sphere_index
                                    && Simulation::did_resolve_rigid_bodies_collision(
                                        current_state,
                                        new_state,
                                        current_sphere_index,
                                        *sphere_index,
                                    )
                                {
                                    // sphere.collision_response.replace(...);
                                }
                            }
                        } else {
                            // Checks neighboring cell.

                            let offset = GridSpaceCoordinate {
                                x: x_offset,
                                y: y_offset,
                                z: z_offset,
                            };

                            let neighbor_coord = current_grid_coord + offset;

                            if let Some(cell) = self.hash_grid.map.get(&neighbor_coord) {
                                for sphere_index in cell {
                                    if Simulation::did_resolve_rigid_bodies_collision(
                                        current_state,
                                        new_state,
                                        current_sphere_index,
                                        *sphere_index,
                                    ) {
                                        // sphere.collision_response.replace(...);
                                    }
                                }
                            }
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
    ) -> bool {
        let s1 = Sphere {
            center: current_state.0[a].position,
            radius: SPHERE_RADIUS,
        };

        let s1_movement = new_state.0[a].position - s1.center;

        let s2 = Sphere {
            center: current_state.0[b].position,
            radius: SPHERE_RADIUS,
        };

        let s2_movement = new_state.0[b].position - s2.center;

        // Narrow-phase collision test on 2 swept spheres.

        // Describes the movement of sphere A from sphere B's frame-of-reference.

        let v = s1_movement - s2_movement;
        let v_distance = v.mag();

        if v_distance.abs() < f32::EPSILON {
            return false;
        }

        match intersect_moving_spheres(s1, s2, v, v_distance) {
            Some((_t, contact_point)) => {
                // Compute and apply the collision response.

                let mut s1_state_cloned = new_state.0[a];
                let mut s2_state_cloned = new_state.0[b];

                resolve_rigid_body_collision(
                    &mut s1_state_cloned,
                    &mut s2_state_cloned,
                    contact_point,
                    &PHYSICS_MATERIAL,
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
            let coord = GridSpaceCoordinate::from((sphere_state, self.hash_grid.scale));

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

    static SPHERE_ROWS: usize = 16;
    static SPHERE_COLUMNS: usize = 16;

    static SPACING: f32 = 4.0;

    static HEIGHT: f32 = SPHERE_ROWS as f32 / 2.0;

    let mut spheres = Vec::with_capacity(SPHERE_ROWS * SPHERE_COLUMNS);

    let grid_offset = Vec3 {
        x: -(SPHERE_ROWS as f32 / 2.0) + SPHERE_RADIUS,
        y: 0.0,
        z: -(SPHERE_COLUMNS as f32 / 2.0) + SPHERE_RADIUS,
    };

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

            let mut sphere =
                RigidBody::new(RigidBodyKind::Sphere(SPHERE_RADIUS), SPHERE_MASS, center);

            let random_velocity = {
                let random_speed = sampler.sample_range_uniform(0.0, 10.0);
                let random_direction = sampler.sample_direction_uniform();

                random_direction * random_speed
            };

            sphere.color = Color::from(&mut *sampler);

            sphere.linear_momentum = random_velocity * sphere.mass;

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
        hash_grid: HashGrid::new(2.0),
    }
}
