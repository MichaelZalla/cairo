use std::f32::consts::TAU;

use cairo::{
    matrix::Mat4,
    physics::{material::PhysicsMaterial, simulation::particle::Particle},
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::strut::Strut;

static STRENGTH_PER_UNIT_LENGTH: f32 = 1750.0;

static DAMPER_PER_UNIT_LENGTH: f32 = 300.0;

#[derive(Default, Debug, Clone)]
pub struct SpringyMesh {
    pub material: PhysicsMaterial,
    pub points: Vec<Particle>,
    pub struts: Vec<Strut>,
    pub state_index_offset: usize,
}

pub fn make_springy_mesh(side_length: f32) -> SpringyMesh {
    let mut sampler: RandomSampler<128> = {
        let mut sampler: RandomSampler<128> = Default::default();

        sampler.seed().unwrap();

        sampler
    };

    let random_speed = sampler.sample_range_normal(5.0, 5.0);

    let random_velocity = sampler.sample_direction_uniform() * random_speed;

    let point_prototype = Particle {
        mass: 10.0,
        velocity: random_velocity,
        ..Default::default()
    };

    // Plots points for uniform triangular prism (tetrahedron).

    let side_length_over_2 = side_length / 2.0;

    let height = side_length * 3.0_f32.sqrt() / 2.0;

    let mut points = vec![
        Particle {
            position: Vec3 {
                x: -side_length_over_2,
                y: 0.0,
                z: -height / 2.0,
            },
            ..point_prototype
        },
        Particle {
            position: Vec3 {
                x: side_length_over_2,
                y: 0.0,
                z: -height / 2.0,
            },
            ..point_prototype
        },
        Particle {
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: height / 2.0,
            },
            ..point_prototype
        },
        Particle {
            position: Vec3 {
                x: 0.0,
                y: side_length * 0.866,
                z: 0.0,
            },
            ..point_prototype
        },
    ];

    // Random mesh transform.

    let random_transform = {
        let random_rotation = {
            let rotate_x =
                Quaternion::new(vec3::RIGHT, sampler.sample_range_normal(0.0, 0.5) * TAU);

            let rotate_y = Quaternion::new(vec3::UP, sampler.sample_range_normal(0.0, 0.5) * TAU);

            let rotate_z =
                Quaternion::new(vec3::FORWARD, sampler.sample_range_normal(0.0, 0.5) * TAU);

            rotate_x * rotate_y * rotate_z
        };

        let random_translation = Mat4::translation(Vec3 {
            x: sampler.sample_range_normal(0.0, 10.0),
            y: sampler.sample_range_normal(10.0, 5.0),
            z: sampler.sample_range_normal(0.0, 10.0),
        });

        *random_rotation.mat() * random_translation
    };

    for point in &mut points {
        point.position = (Vec4::new(point.position, 1.0) * random_transform).to_vec3();
    }

    // Connect points with external struts.

    let rest_length = side_length;

    let prototype = Strut {
        spring_strength: STRENGTH_PER_UNIT_LENGTH / rest_length,
        spring_damper: DAMPER_PER_UNIT_LENGTH / rest_length,
        rest_length,
        ..Default::default()
    };

    let struts = vec![
        // Base (3)
        Strut {
            points: (0, 1),
            ..prototype
        },
        Strut {
            points: (1, 2),
            ..prototype
        },
        Strut {
            points: (0, 2),
            ..prototype
        },
        // Tentpoles (3)
        Strut {
            points: (0, 3),
            ..prototype
        },
        Strut {
            points: (1, 3),
            ..prototype
        },
        Strut {
            points: (2, 3),
            ..prototype
        },
    ];

    // Random physics material.

    let material = {
        let random_friction = sampler.sample_range_normal(0.25, 0.1).clamp(0.15, 1.0);
        let random_restitution = sampler.sample_range_normal(0.9, 0.2).clamp(0.0, 1.0);

        PhysicsMaterial {
            dynamic_friction: random_friction,
            restitution: random_restitution,
        }
    };

    SpringyMesh {
        points,
        struts,
        material,
        state_index_offset: 0,
    }
}
