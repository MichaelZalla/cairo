use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use cairo::{
    physics::simulation::{force::ContactPoint, physical_constants::EARTH_GRAVITY, units::Newtons},
    random::sampler::RandomSampler,
    vec::vec3::Vec3,
};

use crate::{
    particle::{
        generator::{ParticleGenerator, ParticleGeneratorKind},
        PARTICLE_MASS,
    },
    simulation::{Operators, ParticleForce, Simulation},
    state_vector::StateVector,
};

pub(crate) const SEED_SIZE: usize = 2048;

static GRAVITY: ParticleForce =
    |_state: &StateVector, _i: usize, _current_time: f32| -> (Newtons, Option<ContactPoint>) {
        let newtons = Vec3 {
            x: 0.0,
            y: -EARTH_GRAVITY,
            z: 0.0,
        };

        (newtons, None)
    };

static AIR_RESISTANCE: ParticleForce =
    |state: &StateVector, i: usize, _current_time: f32| -> (Newtons, Option<ContactPoint>) {
        static D: f32 = 0.0;

        static WIND: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let newtons = (WIND - state.data[i + state.len()]) * D;

        (newtons, None)
    };

pub(crate) fn make_simulation<'a>(
    sampler: Rc<RefCell<RandomSampler<SEED_SIZE>>>,
    _sampler_for_random_acceleration_operator: Rc<RefCell<RandomSampler<SEED_SIZE>>>,
) -> Simulation<'a, SEED_SIZE> {
    let mass = PARTICLE_MASS;

    // Define some particle generators.

    let omnidirectional = ParticleGenerator::new(
        ParticleGeneratorKind::Omnidirectional(Vec3 {
            x: 0.0,
            y: 40.0,
            z: 0.0,
        }),
        50.0,
        None,
        mass,
        15.0,
        5.0,
    );

    let directional_right = ParticleGenerator::new(
        ParticleGeneratorKind::Directed(
            Vec3 {
                x: -75.0,
                y: 30.0,
                z: 0.0,
            },
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        50.0,
        Some(PI / 4.0),
        mass,
        15.0,
        5.0,
    );

    let directional_up = ParticleGenerator::new(
        ParticleGeneratorKind::Directed(
            Vec3 {
                x: 75.0,
                y: 30.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.001,
                y: 1.0,
                z: 0.0,
            },
        ),
        50.0,
        Some(PI / 2.0),
        mass,
        15.0,
        5.0,
    );

    let operators = Operators {
        additive_acceleration: vec![
            // Additive acceleration operator: Contributes a random acceleration.
            // Box::new(
            //     move |_current_state: &StateVector,
            //           _i: usize,
            //           _total_acceleration: &Acceleration,
            //           h: f32|
            //           -> Acceleration {
            //         static SCALING_FACTOR: f32 = 1.0;

            //         let mut sampler = sampler_for_random_acceleration_operator.borrow_mut();

            //         sampler.sample_direction_uniform() * SCALING_FACTOR / h
            //     },
            // ),
            // Additive acceleration operator: Avoids a static sphere collider.
            // Box::new(
            //     |current_state: &StateVector,
            //      i: usize,
            //      total_acceleration: &Acceleration,
            //      _h: f32|
            //      -> Acceleration {
            //         static COLLIDER_CENTER: Vec3 = Vec3 {
            //             x: -15.0,
            //             y: -15.0,
            //             z: 0.0,
            //         };

            //         static COLLIDER_RADIUS: f32 = 40.0;
            //         static COLLIDER_SAFE_RADIUS: f32 = COLLIDER_RADIUS + 5.0;
            //         static THRESHOLD_TIME: f32 = 3.0;

            //         let n = current_state.len();

            //         let current_position = current_state.data[i];
            //         let current_velocity = current_state.data[i + n];

            //         if current_velocity.mag() == 0.0 {
            //             return Acceleration::default();
            //         }

            //         let particle_direction = current_velocity.as_normal();

            //         let particle_to_collider_center = COLLIDER_CENTER - current_position;

            //         let distance_of_particle_closest_approach_to_collider_center =
            //             particle_to_collider_center.dot(particle_direction);

            //         if distance_of_particle_closest_approach_to_collider_center < 0.0 {
            //             return Acceleration::default();
            //         }

            //         let distance_of_concern = current_velocity.mag() * THRESHOLD_TIME;

            //         if distance_of_particle_closest_approach_to_collider_center
            //             > distance_of_concern
            //         {
            //             return Acceleration::default();
            //         }

            //         let closest_approach = current_position
            //             + particle_direction
            //                 * distance_of_particle_closest_approach_to_collider_center;

            //         let collider_center_to_closest_approach = closest_approach - COLLIDER_CENTER;

            //         let collider_center_to_closest_approach_direction =
            //             collider_center_to_closest_approach.as_normal();

            //         let collider_center_to_closest_approach_distance =
            //             collider_center_to_closest_approach.mag();

            //         if collider_center_to_closest_approach_distance > COLLIDER_SAFE_RADIUS {
            //             return Acceleration::default();
            //         }

            //         let turning_target = COLLIDER_CENTER
            //             + collider_center_to_closest_approach_direction * COLLIDER_SAFE_RADIUS;

            //         let particle_to_turning_target = turning_target - current_position;
            //         let particle_to_turning_target_distance = particle_to_turning_target.mag();

            //         let velocity_towards_turning_target = current_velocity
            //             .dot(particle_to_turning_target / particle_to_turning_target_distance);

            //         let time_to_reach_turning_target =
            //             particle_to_turning_target_distance / velocity_towards_turning_target;

            //         let average_speed_increase_othogonal_to_velocity =
            //             (particle_direction.cross(particle_to_turning_target)).mag()
            //                 / time_to_reach_turning_target;

            //         let required_magnitude_of_acceleration = 2.0
            //             * average_speed_increase_othogonal_to_velocity
            //             / time_to_reach_turning_target;

            //         let existing_acceleration_in_collider_center_to_closest_approach_direction =
            //             collider_center_to_closest_approach_direction.dot(*total_acceleration);

            //         collider_center_to_closest_approach_direction * (required_magnitude_of_acceleration
            //         - existing_acceleration_in_collider_center_to_closest_approach_direction)
            //         .max(0.0)
            //     },
            // ),
        ],
        functional_acceleration: vec![
            // Functional acceleration operator: Enforces a minimum velocity;
            // Box::new(
            //     |current_state: &StateVector,
            //      i: usize,
            //      new_velocity: &Velocity,
            //      _h: f32|
            //      -> Velocity {
            //         static MINIMUM_SPEED: f32 = 20.0;

            //         let n = current_state.len();

            //         let current_velocity = current_state.data[i + n];

            //         let current_speed = current_velocity.mag();
            //         let new_speed = new_velocity.mag();

            //         if new_speed >= MINIMUM_SPEED {
            //             return *new_velocity;
            //         }

            //         if current_speed > MINIMUM_SPEED {
            //             current_velocity
            //         } else {
            //             current_velocity.as_normal() * MINIMUM_SPEED
            //         }
            //     },
            // ),
            // Functional acceleration operator: Rotation around the Z-axis.
            // Box::new(
            //     |_current_state: &StateVector,
            //      _i: usize,
            //      new_velocity: &Velocity,
            //      h: f32|
            //      -> Velocity {
            //         static ANGLE: f32 = PI / 2.0;

            //         *new_velocity * Mat4::rotation_z(ANGLE * h)
            //     },
            // ),
        ],
        velocity: vec![
            // Velocity operator: Vortex.
            // Box::new(
            //     |current_state: &StateVector,
            //      i: usize,
            //      new_velocity: &Velocity,
            //      h: f32|
            //      -> Velocity {
            //         static VORTEX_CENTER: Vec3 = Vec3 {
            //             x: 0.0,
            //             y: 0.0,
            //             z: 0.0,
            //         };

            //         static VORTEX_RADIUS: f32 = 200.0;

            //         static VORTEX_ROTATIONAL_FREQUENCY_AT_RADIUS: f32 = 1.5;

            //         static VORTEX_ROTATIONAL_FREQUENCY_MAX: f32 = 10.0;

            //         static VORTEX_TIGHTNESS: f32 = 1.5;

            //         let current_position = current_state.data[i];

            //         let particle_distance_to_vortex_center =
            //             (current_position - VORTEX_CENTER).mag();

            //         let particle_rotational_frequency_scaling_factor =
            //             (VORTEX_RADIUS / particle_distance_to_vortex_center).powf(VORTEX_TIGHTNESS);

            //         let particle_rotational_frequency = (VORTEX_ROTATIONAL_FREQUENCY_AT_RADIUS
            //             * particle_rotational_frequency_scaling_factor)
            //             .max(VORTEX_ROTATIONAL_FREQUENCY_MAX);

            //         let omega = TAU * particle_rotational_frequency;

            //         *new_velocity * Mat4::rotation_z(omega * h)
            //     },
            // ),
            // Velocity operator: Translation by offset.
            // Box::new(
            //     |_current_state: &StateVector,
            //      _i: usize,
            //      new_velocity: &Velocity,
            //      _h: f32|
            //      -> Velocity {
            //         static OFFSET: Vec3 = Vec3 {
            //             x: 50.0,
            //             y: 0.0,
            //             z: 0.0,
            //         };

            //         *new_velocity + OFFSET
            //     },
            // ),
        ],
    };

    Simulation {
        sampler,
        pool: Default::default(),
        forces: vec![&GRAVITY, &AIR_RESISTANCE],
        colliders: Default::default(),
        operators: RefCell::new(operators),
        generators: RefCell::new(vec![omnidirectional, directional_right, directional_up]),
        quadtree: RefCell::new(Default::default()),
    }
}
