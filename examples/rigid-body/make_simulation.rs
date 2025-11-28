use cairo::{
    color::{self, Color},
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

    // Test 2: Spinning on Y-axis (should spin in place)
    {
        let center = Vec3 {
            x: -3.5 * SPACING,
            y: RADIUS,
            z: 0.0, // Back in collision path with Red
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::MAGENTA;
        sphere.angular_momentum = vec3::UP * 1.5;

        spheres.push(sphere);
    }

    // Test 3a: Rolling toward viewer (Z-)
    {
        let center = Vec3 {
            x: -1.5 * SPACING,
            y: RADIUS,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::BLUE;
        sphere.angular_momentum = vec3::RIGHT * 1.5;

        // Rolling toward viewer: angular momentum +X → linear velocity -Z
        // ω = -3.75 (after negation), v = (0, 0, -3.75)
        sphere.linear_momentum = vec3::FORWARD * MASS * -3.75;

        spheres.push(sphere);
    }

    // Test 3b: Rolling away from viewer (Z+)
    {
        let center = Vec3 {
            x: -0.5 * SPACING,
            y: RADIUS,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::GREEN;
        sphere.angular_momentum = -vec3::RIGHT * 1.5;

        // Rolling away from viewer: angular momentum -X → linear velocity +Z
        // ω = +3.75 (after negation of -1.5), v = (0, 0, +3.75)
        sphere.linear_momentum = vec3::FORWARD * MASS * 3.75;

        spheres.push(sphere);
    }

    // Test 3c: Rolling to the right (X+)
    // Rotate Blue's setup: X→Z, Z→X, then negate both to get +X motion
    {
        let center = Vec3 {
            x: 0.5 * SPACING,
            y: RADIUS,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::RED;
        // Rotate Blue 90° CCW: X→Z, Z→-X
        // Blue: L = +X, v = -Z
        // Red:  L = +Z, v = +X
        sphere.angular_momentum = vec3::FORWARD * 1.5;
        sphere.linear_momentum = vec3::RIGHT * MASS * 3.75;

        spheres.push(sphere);
    }

    // Test 3d: Rolling to the left (X-)
    // Rotate Green 90° CCW: X→Z, Z→-X
    {
        let center = Vec3 {
            x: 1.5 * SPACING,
            y: RADIUS,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::YELLOW;
        // Green: L = -X, v = +Z
        // Yellow: L = -Z, v = -X
        sphere.angular_momentum = -vec3::FORWARD * 1.5;
        sphere.linear_momentum = -vec3::RIGHT * MASS * 3.75;

        spheres.push(sphere);
    }

    // Test 4: Dropping from height
    {
        let center = Vec3 {
            x: 1.0 * SPACING,
            y: 10.0,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::CYAN;

        spheres.push(sphere);
    }

    // Test 5: Dropping with X-velocity
    {
        let center = Vec3 {
            x: 2.0 * SPACING,
            y: 10.0,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::WHITE;
        sphere.linear_momentum = vec3::RIGHT * MASS * 5.0;

        spheres.push(sphere);
    }

    // Test 6: Dropping with Z-velocity
    {
        let center = Vec3 {
            x: 3.0 * SPACING,
            y: 10.0,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = color::DARK_GRAY;
        sphere.linear_momentum = vec3::FORWARD * MASS * 5.0;

        spheres.push(sphere);
    }

    // Test 7: Diagonal rolling (X+/Z+, northeast direction)
    {
        let center = Vec3 {
            x: 2.5 * SPACING,
            y: RADIUS,
            z: -2.0 * SPACING,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = Color::rgb(255, 165, 0); // Orange-ish (distinct from ORANGE)

        // For diagonal rolling at 45°: combine angular momentum components
        // Angular momentum should be perpendicular to motion direction
        // Motion: +X +Z, so angular momentum: -X +Z (rotated 90° CCW in XZ plane)
        let diagonal_angular = (vec3::FORWARD - vec3::RIGHT) * 1.5;
        sphere.angular_momentum = diagonal_angular;

        // Linear velocity matches rolling constraint: v = ω × r
        let diagonal_velocity = (vec3::RIGHT + vec3::FORWARD).as_normal() * 3.75;
        sphere.linear_momentum = diagonal_velocity * MASS;

        spheres.push(sphere);
    }

    // Test 8: Dropping with diagonal velocity (X+/Z+)
    {
        let center = Vec3 {
            x: -2.0 * SPACING,
            y: 15.0,
            z: -2.0 * SPACING,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = Color::rgb(255, 192, 203); // Pink

        // Diagonal horizontal velocity
        let diagonal_velocity = (vec3::RIGHT + vec3::FORWARD).as_normal() * 5.0;
        sphere.linear_momentum = diagonal_velocity * MASS;

        spheres.push(sphere);
    }

    // Test 9: High drop test
    {
        let center = Vec3 {
            x: -3.0 * SPACING,
            y: 30.0,
            z: -2.0 * SPACING,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = Color::rgb(128, 0, 128); // Purple

        spheres.push(sphere);
    }

    // Test 10: Very high drop test
    {
        let center = Vec3 {
            x: -4.0 * SPACING,
            y: 50.0,
            z: -2.0 * SPACING,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = Color::rgb(165, 42, 42); // Brown

        spheres.push(sphere);
    }

    // Test 11: Fast spinning (high-energy backspin)
    {
        let center = Vec3 {
            x: -4.5 * SPACING,
            y: RADIUS,
            z: 0.0,
        };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(RADIUS), MASS, center);

        sphere.color = Color::rgb(0, 255, 255); // Bright cyan
        sphere.angular_momentum = vec3::UP * 5.0; // Much faster than Test 2

        spheres.push(sphere);
    }

    for sphere in spheres.iter_mut() {
        sphere.debug_flags |= RigidBodyDebugFlags::DRAW_LINEAR_VELOCITY
        | RigidBodyDebugFlags::DRAW_ANGULAR_VELOCITY
        // | RigidBodyDebugFlags::DRAW_COLLISION_CONTACT_POINT
        // RigidBodyDebugFlags::DRAW_COLLISION_CONTACT_POINT_VELOCITY
        // | RigidBodyDebugFlags::DRAW_COLLISION_FRICTION_IMPULSE
        // | RigidBodyDebugFlags::DRAW_COLLISION_NORMAL_IMPULSE
        // | RigidBodyDebugFlags::DRAW_COLLISION_TANGENT
        | RigidBodyDebugFlags::DRAW_STATIC_CONTACT
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
