use std::f32::consts::TAU;

use cairo::{
    color,
    matrix::Mat4,
    physics::{material::PhysicsMaterial, simulation::particle::Particle},
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::strut::{Edge, Strut, PARTICLE_MASS};

#[derive(Default, Debug, Clone)]
pub struct SpringyMesh {
    pub material: PhysicsMaterial,
    pub points: Vec<Particle>,
    pub struts: Vec<Strut>,
    pub state_index_offset: usize,
}

#[allow(unused)]
pub fn make_tetrahedron(side_length: f32) -> (Vec<Particle>, Vec<Strut>) {
    // Plots points for uniform triangular prism (tetrahedron).

    let side_length_over_2 = side_length / 2.0;

    let height = side_length * 3.0_f32.sqrt() / 2.0;

    let vertices = vec![
        Vec3 {
            x: -side_length_over_2,
            y: 0.0,
            z: -height / 2.0,
        },
        Vec3 {
            x: side_length_over_2,
            y: 0.0,
            z: -height / 2.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: height / 2.0,
        },
        Vec3 {
            x: 0.0,
            y: side_length * 0.866,
            z: 0.0,
        },
    ];

    // Connect points with edges.

    let edge_data = vec![
        // Base (3)
        (0, 1, color::LIGHT_GRAY),
        (1, 2, color::LIGHT_GRAY),
        (0, 2, color::LIGHT_GRAY),
        // Tentpoles (3)
        (0, 3, color::LIGHT_GRAY),
        (1, 3, color::LIGHT_GRAY),
        (2, 3, color::LIGHT_GRAY),
    ];

    let edges = edge_data
        .into_iter()
        .map(|data| Edge {
            points: (data.0, data.1),
            color: data.2,
        })
        .collect();

    make_points_and_struts(vertices, edges)
}

#[allow(unused)]
pub fn make_cube(side_length: f32) -> (Vec<Particle>, Vec<Strut>) {
    // Plots points for a cube.

    let front = vec![
        // Top left
        Vec3 {
            x: -0.5,
            y: 0.5,
            z: -0.5,
        },
        // Top right
        Vec3 {
            x: 0.5,
            y: 0.5,
            z: -0.5,
        },
        // Bottom right
        Vec3 {
            x: 0.5,
            y: -0.5,
            z: -0.5,
        },
        // Bottom left
        Vec3 {
            x: -0.5,
            y: -0.5,
            z: -0.5,
        },
    ];

    let back: Vec<Vec3> = front.iter().map(|c| Vec3 { z: 0.5, ..*c }).collect();

    let vertices: Vec<Vec3> = front
        .into_iter()
        .chain(back)
        .map(|v| v * side_length)
        .collect();

    // Connect points with external struts.

    let edge_data = vec![
        // Front loop (4)
        (0, 1, color::RED),
        (1, 2, color::RED),
        (2, 3, color::RED),
        (3, 0, color::RED),
        // Back loop (4)
        (4, 5, color::BLUE),
        (5, 6, color::BLUE),
        (6, 7, color::BLUE),
        (7, 4, color::BLUE),
        // Front-to-back connections (4)
        (0, 4, color::YELLOW),
        (1, 5, color::YELLOW),
        (2, 6, color::YELLOW),
        (3, 7, color::YELLOW),
        // Cross-face struts (6)
        (0, 2, color::LIGHT_GRAY),
        (1, 6, color::LIGHT_GRAY),
        (5, 7, color::LIGHT_GRAY),
        (4, 3, color::LIGHT_GRAY),
        (4, 1, color::LIGHT_GRAY),
        (3, 6, color::LIGHT_GRAY),
        // Internal struts (2)
        (0, 6, color::DARK_GRAY),
        (5, 3, color::DARK_GRAY),
    ];

    let edges = edge_data
        .into_iter()
        .map(|data| Edge {
            points: (data.0, data.1),
            color: data.2,
        })
        .collect();

    make_points_and_struts(vertices, edges)
}

fn make_points_and_struts(vertices: Vec<Vec3>, edges: Vec<Edge>) -> (Vec<Particle>, Vec<Strut>) {
    let points: Vec<Particle> = vertices
        .into_iter()
        .map(|position| Particle {
            position,
            ..Default::default()
        })
        .collect();

    let struts: Vec<Strut> = edges
        .into_iter()
        .map(|edge| Strut::new(&points, edge))
        .collect();

    (points, struts)
}

pub fn make_springy_mesh(mut points: Vec<Particle>, struts: Vec<Strut>) -> SpringyMesh {
    let mut sampler: RandomSampler<128> = {
        let mut sampler: RandomSampler<128> = Default::default();

        sampler.seed().unwrap();

        sampler
    };

    let random_speed = sampler.sample_range_normal(5.0, 5.0);

    let random_velocity = sampler.sample_direction_uniform() * random_speed;

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
        point.velocity = random_velocity;
        point.mass = PARTICLE_MASS;
    }

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
