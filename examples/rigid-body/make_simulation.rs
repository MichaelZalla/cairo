use std::f32::consts::PI;

use cairo::{
    color::Color,
    geometry::accelerator::hash_grid::{HashGrid, HashGridInsertionStrategy},
    physics::simulation::{
        force::gravity::GRAVITY_RIGID_BODY_FORCE,
        rigid_body::{rigid_body_simulation_state::DynRigidBodyForce, RigidBody, RigidBodyKind},
    },
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

use crate::{plane_collider::PlaneCollider, simulation::Simulation};

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
