use std::f32::consts::PI;

use cairo::{
    color::{self, Color},
    geometry::{
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

use crate::{
    hash_grid::{GridSpaceCoordinate, HashGrid},
    integration::system_dynamics_function,
};

use crate::{plane_collider::PlaneCollider, state_vector::StateVector};

static SPHERE_RADIUS: f32 = 0.5;
static SPHERE_MASS: f32 = 1.0;

static PHYSICS_MATERIAL: PhysicsMaterial = PhysicsMaterial {
    dynamic_friction: 0.0,
    restitution: 0.84,
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
            state.0[i] = sphere.state();
        }

        let derivative = system_dynamics_function(&state, &self.forces, uptime_seconds);

        // Performs basic forward Euler integration over position and velocity.

        let mut new_state = state.clone() + derivative.clone() * h;

        for sphere in self.rigid_bodies.iter_mut() {
            sphere.did_collide = false;
        }

        // Detects and resolves collisions with static colliders.

        self.check_static_collisions(&state, &mut new_state);

        // Detects and resolves collisions with other (nearby) rigid bodies.

        self.rebuild_hash_grid(&new_state);

        self.check_rigid_body_collisions(&state, &mut new_state);

        // Copies new state back to rigid bodies.

        for (i, sphere) in self.rigid_bodies.iter_mut().enumerate() {
            sphere.apply_simulation_state(&new_state.0[i]);
        }
    }

    fn check_static_collisions(
        &mut self,
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
            let end_position = new_body_state.position;
            let end_linear_momentum = new_body_state.linear_momentum;
            let end_velocity = end_linear_momentum * new_body_state.inverse_mass;

            for collider in &self.static_plane_colliders {
                if end_velocity.dot(collider.plane.normal) > 0.0 {
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

                    let state = &mut new_state.0[i];

                    resolve_rigid_body_plane_collision(
                        state,
                        collider.plane.normal,
                        contact_point,
                        &PHYSICS_MATERIAL,
                    );

                    sphere.did_collide = true;
                }
            }
        }
    }

    fn check_rigid_body_collisions(
        &mut self,
        current_state: &StateVector<RigidBodySimulationState>,
        new_state: &mut StateVector<RigidBodySimulationState>,
    ) {
        for (current_sphere_index, sphere) in self.rigid_bodies.iter_mut().enumerate() {
            let sphere_state = &new_state.0[current_sphere_index];

            let current_grid_coord = GridSpaceCoordinate::from(sphere_state);

            for x_offset in -1..=1 {
                for y_offset in -1..=1 {
                    for z_offset in -1..=1 {
                        if x_offset == 0 && y_offset == 0 && z_offset == 0 {
                            // Checks current cell.

                            let cell = self.hash_grid.get(&current_grid_coord).unwrap();

                            for sphere_index in cell {
                                if *sphere_index != current_sphere_index
                                    && Simulation::did_resolve_collision(
                                        current_state,
                                        new_state,
                                        current_sphere_index,
                                        *sphere_index,
                                    )
                                {
                                    sphere.did_collide = true;
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

                            if let Some(cell) = self.hash_grid.get(&neighbor_coord) {
                                for sphere_index in cell {
                                    if Simulation::did_resolve_collision(
                                        current_state,
                                        new_state,
                                        current_sphere_index,
                                        *sphere_index,
                                    ) {
                                        sphere.did_collide = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn did_resolve_collision(
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

        match intersect_moving_spheres(s1, s1_movement, s2, s2_movement) {
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
        self.hash_grid.clear();

        for (i, sphere_state) in new_state.0.iter().enumerate() {
            let coord = GridSpaceCoordinate::from(sphere_state);

            match self.hash_grid.get_mut(&coord) {
                Some(list) => {
                    list.push(i);
                }
                None => {
                    let mut cell = Vec::<usize>::with_capacity(4);

                    cell.insert(0, i);

                    self.hash_grid.insert(coord, cell);
                }
            }
        }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        // Render dynamic rigid bodies (spheres).

        for sphere in &self.rigid_bodies {
            // Visualize rigid body AABB.

            if let Some(aabb) = &sphere.aabb {
                renderer.render_aabb(aabb, Default::default(), color::DARK_GRAY);
            }

            let transform = &sphere.transform;

            let radius = match sphere.kind {
                RigidBodyKind::Sphere(radius) => radius,
                _ => panic!(),
            };

            let transform_with_radius = Mat4::scale_uniform(radius) * *transform.mat();

            let display_kind = EmptyDisplayKind::Sphere(12);

            let color = if sphere.did_collide {
                color::RED
            } else {
                sphere.color
            };

            renderer.render_empty(&transform_with_radius, display_kind, true, Some(color));
        }

        // Visualize hash grid entries.

        for grid_coord in self.hash_grid.keys() {
            let transform = {
                let scale = Mat4::scale_uniform(0.5);

                let offset = Mat4::translation(vec3::ONES * 0.5);

                let translate = Mat4::translation(grid_coord.into());

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
                y: 0.0,
                z: 0.0,
            },
            Quaternion::new(vec3::FORWARD, PI / 12.0),
        ),
        (
            Vec3 {
                x: 20.0,
                y: 0.0,
                z: 0.0,
            },
            Quaternion::new(vec3::FORWARD, -PI / 12.0),
        ),
        (
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -30.0,
            },
            Quaternion::new(vec3::RIGHT, -PI / 12.0),
        ),
        (
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 30.0,
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
        hash_grid: Default::default(),
    }
}
