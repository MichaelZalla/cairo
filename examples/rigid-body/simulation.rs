use std::f32::consts::PI;

use cairo::{
    color,
    geometry::intersect::intersect_capsule_plane,
    matrix::Mat4,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            collision_response::resolve_rigid_body_plane_collision,
            force::gravity::GRAVITY_RIGID_BODY_FORCE,
            rigid_body::{
                rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
                RigidBody, RigidBodyKind,
            },
        },
    },
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

use crate::integration::system_dynamics_function;

use crate::{plane_collider::PlaneCollider, state_vector::StateVector};

static SPHERE_RADIUS: f32 = 0.5;

pub struct Simulation {
    pub forces: Vec<Box<DynRigidBodyForce>>,
    pub rigid_bodies: Vec<RigidBody>,
    pub static_plane_colliders: Vec<PlaneCollider>,
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

        // Detects and resolves collisions with static colliders.

        for i in 0..n {
            let sphere = &self.rigid_bodies[i];

            let radius = match sphere.kind {
                RigidBodyKind::Sphere(radius) => radius,
                _ => panic!(),
            };

            let linear_acceleration = derivative.0[i].linear_momentum;

            let old_body_state = &state.0[i];
            let new_body_state = &mut new_state.0[i];

            let mass = 1.0 / new_body_state.inverse_mass;

            let start_position = old_body_state.position;
            let end_position = new_body_state.position;
            let end_linear_momentum = new_body_state.linear_momentum;
            let end_velocity = end_linear_momentum * new_body_state.inverse_mass;

            for collider in &self.static_plane_colliders {
                if end_velocity.dot(collider.plane.normal) > 0.0 {
                    // The sphere is moving away from the plane, so no collision could occur.
                    continue;
                }

                if let Some((f, contact_point)) =
                    intersect_capsule_plane(start_position, end_position, radius, &collider.plane)
                {
                    if f > 1.0 {
                        // Ignores potential (future) intersection.
                        continue;
                    }

                    let time_before_collision = h * f;
                    let time_after_collision = h - time_before_collision;

                    let accumulated_linear_velocity =
                        linear_acceleration * 2.0 * time_after_collision;

                    new_state.0[i].linear_momentum -= accumulated_linear_velocity * mass;

                    new_state.0[i].position = start_position
                        + (end_position - start_position) * f
                        + collider.plane.normal * 0.001;

                    static PHYSICS_MATERIAL: PhysicsMaterial = PhysicsMaterial {
                        dynamic_friction: 0.0,
                        restitution: 0.76,
                    };

                    let state = &mut new_state.0[i];

                    resolve_rigid_body_plane_collision(
                        state,
                        collider.plane.normal,
                        contact_point,
                        &PHYSICS_MATERIAL,
                    );
                }
            }
        }

        // Copies new state back to rigid bodies.

        for (i, sphere) in self.rigid_bodies.iter_mut().enumerate() {
            sphere.apply_simulation_state(&new_state.0[i]);
        }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        for sphere in &self.rigid_bodies {
            let transform = &sphere.transform;

            let radius = match sphere.kind {
                RigidBodyKind::Sphere(radius) => radius,
                _ => panic!(),
            };

            let transform_with_radius = Mat4::scale_uniform(radius) * *transform.mat();

            let display_kind = EmptyDisplayKind::Sphere(12);

            renderer.render_empty(
                &transform_with_radius,
                display_kind,
                true,
                Some(color::WHITE),
            );
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

pub fn make_simulation() -> Simulation {
    // Forces.

    let forces: Vec<Box<DynRigidBodyForce>> = vec![Box::new(GRAVITY_RIGID_BODY_FORCE)];

    // Rigid bodies (spheres).

    static SPHERE_ROWS: usize = 8;
    static SPHERE_COLUMNS: usize = 8;

    static SPACING: f32 = 5.0;
    static HEIGHT: f32 = 3.0;

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

            let sphere = RigidBody::new(RigidBodyKind::Sphere(SPHERE_RADIUS), 1.0, center);

            spheres.push(sphere);
        }
    }

    // Ground (collider) planes.

    let mut static_plane_colliders = vec![];

    let ground_planes = vec![
        (
            Vec3 {
                x: -10.0,
                y: 0.0,
                z: 0.0,
            },
            Quaternion::new(vec3::FORWARD, PI / 12.0),
        ),
        (
            Vec3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            Quaternion::new(vec3::FORWARD, -PI / 12.0),
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
    }
}
