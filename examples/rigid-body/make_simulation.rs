use cairo::{
    color,
    geometry::accelerator::hash_grid::{HashGrid, HashGridInsertionStrategy},
    physics::simulation::{
        force::gravity::GRAVITY_RIGID_BODY_FORCE,
        rigid_body::{
            RigidBody, RigidBodyDebugFlags, RigidBodyKind,
            rigid_body_simulation_state::DynRigidBodyForce,
        },
    },
    random::sampler::RandomSampler,
    vec::vec3::{self, Vec3},
};

use crate::{plane_collider::PlaneCollider, simulation::Simulation};

pub fn make_simulation(_sampler: &mut RandomSampler<1024>) -> Simulation {
    // Forces.

    let forces: Vec<Box<DynRigidBodyForce>> = vec![
        // Gravity
        Box::new(GRAVITY_RIGID_BODY_FORCE),
    ];

    // Rigid bodies (spheres) - test configurations.

    static RADIUS: f32 = 1.0;
    static MASS: f32 = 1.0;
    static SPACING: f32 = 4.0;

    let mut spheres = Vec::new();

    // Test 1: Static on ground
    {
        let center = Vec3 {
            x: 3.0 * SPACING,
            y: RADIUS,
            z: -3.0 * SPACING, // Position away from other spheres
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::ORANGE; // Use orange to distinguish from Red sphere

        spheres.push(sphere);
    }

    for sphere in spheres.iter_mut() {
        sphere.debug_flags |=
            RigidBodyDebugFlags::DRAW_LINEAR_VELOCITY | RigidBodyDebugFlags::DRAW_ANGULAR_VELOCITY
        // | RigidBodyDebugFlags::DRAW_COLLISION_CONTACT_POINT
        // | RigidBodyDebugFlags::DRAW_COLLISION_CONTACT_POINT_VELOCITY
        // | RigidBodyDebugFlags::DRAW_COLLISION_FRICTION_IMPULSE
        // | RigidBodyDebugFlags::DRAW_COLLISION_NORMAL_IMPULSE
        // | RigidBodyDebugFlags::DRAW_COLLISION_TANGENT
        // | RigidBodyDebugFlags::DRAW_STATIC_CONTACT
    }

    // Ground (collider) plane - single flat plane at y=0.

    let static_plane_colliders = vec![
        PlaneCollider::new(Vec3::default(), vec3::UP), // Flat ground at origin, facing up
    ];

    Simulation {
        forces,
        rigid_bodies: spheres,
        static_plane_colliders,
        hash_grid: HashGrid::new(HashGridInsertionStrategy::default(), RADIUS * 2.0),
        visualize_hash_grid: false,
        tick_count: 0,
    }
}
