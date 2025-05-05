use std::{collections::HashMap, f32::consts::TAU};

use cairo::{
    color,
    geometry::primitives::{aabb::AABB, triangle::Triangle},
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
    pub triangles: Vec<Triangle>,
    pub aabb: AABB,
}

impl SpringyMesh {
    pub fn update_aabb(&mut self) {
        self.aabb = AABB::default();

        for point in &self.points {
            self.aabb.grow(&point.position);
        }

        self.aabb.recompute_derived_state();
    }
}

#[allow(unused)]
pub fn make_tetrahedron(side_length: f32) -> (Vec<Particle>, Vec<Strut>) {
    // Plots points for a uniform triangular prism (tetrahedron).

    let f = 2.0 * 2.0_f32.sqrt();

    let factor = side_length / f;

    // Embeds an equilateral triangular pyramid inside of a cube.
    // See: https://en.wikipedia.org/wiki/Tetrahedron#Cartesian_coordinates
    let vertices = vec![
        // Base (3)
        Vec3 {
            x: -factor,
            y: factor,
            z: -factor,
        },
        Vec3 {
            x: factor,
            y: -factor,
            z: -factor,
        },
        Vec3 {
            x: -factor,
            y: -factor,
            z: factor,
        },
        // Top (1)
        Vec3 {
            x: factor,
            y: factor,
            z: factor,
        },
    ];

    // Connect points with edges.

    let edge_data = vec![
        // Base (3)
        (0, 1, 3, 2, color::LIGHT_GRAY),
        (1, 2, 3, 0, color::LIGHT_GRAY),
        (0, 2, 1, 3, color::LIGHT_GRAY),
        // Tentpoles (3)
        (0, 3, 2, 1, color::LIGHT_GRAY),
        (1, 3, 0, 2, color::LIGHT_GRAY),
        (2, 3, 1, 0, color::LIGHT_GRAY),
    ];

    let edges = edge_data
        .into_iter()
        .map(|data| Edge {
            points: (data.0, data.1),
            connected_points: Some((data.2, data.3)),
            color: data.4,
        })
        .collect();

    make_points_and_struts(vertices, edges)
}

#[allow(unused)]
pub fn make_cube(side_length: f32) -> (Vec<Particle>, Vec<Strut>) {
    // Plots points for a cube.

    let side_length_over_2 = side_length / 2.0;

    let front = vec![
        // Top left
        Vec3 {
            x: -side_length_over_2,
            y: side_length_over_2,
            z: -side_length_over_2,
        },
        // Top right
        Vec3 {
            x: side_length_over_2,
            y: side_length_over_2,
            z: -side_length_over_2,
        },
        // Bottom right
        Vec3 {
            x: side_length_over_2,
            y: -side_length_over_2,
            z: -side_length_over_2,
        },
        // Bottom left
        Vec3 {
            x: -side_length_over_2,
            y: -side_length_over_2,
            z: -side_length_over_2,
        },
    ];

    let mut back = front.clone();

    for v in back.iter_mut() {
        v.z = side_length_over_2;
    }

    let vertices: Vec<Vec3> = front.into_iter().chain(back).collect();

    // Connect points with external struts.

    let edge_data = vec![
        // Front loop (4)
        (0, 1, 4, 2, color::RED),
        (1, 2, 6, 0, color::RED),
        (2, 3, 6, 0, color::RED),
        (3, 0, 4, 2, color::RED),
        // Back loop (4)
        (4, 5, 7, 1, color::RED),
        (5, 6, 7, 1, color::RED),
        (6, 7, 5, 3, color::RED),
        (7, 4, 5, 3, color::RED),
        // Front-to-back connections (4)
        (0, 4, 3, 1, color::RED),
        (1, 5, 4, 6, color::RED),
        (2, 6, 1, 3, color::RED),
        (3, 7, 6, 4, color::RED),
        // Cross-face struts (6)
        (0, 2, 1, 3, color::DARK_GRAY),
        (1, 6, 5, 2, color::DARK_GRAY),
        (5, 7, 4, 6, color::DARK_GRAY),
        (4, 3, 0, 7, color::DARK_GRAY),
        (4, 1, 5, 0, color::DARK_GRAY),
        (3, 6, 2, 7, color::DARK_GRAY),
        // Internal struts (2)
        (0, 6, usize::MAX, usize::MAX, color::DARK_GRAY),
        (5, 3, usize::MAX, usize::MAX, color::DARK_GRAY),
    ];

    let edges = edge_data
        .into_iter()
        .map(|data| Edge {
            points: (data.0, data.1),
            connected_points: if data.2 == usize::MAX {
                None
            } else {
                Some((data.2, data.3))
            },
            color: data.4,
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

pub fn make_springy_mesh(
    mut points: Vec<Particle>,
    struts: Vec<Strut>,
    sampler: &mut RandomSampler<1024>,
) -> SpringyMesh {
    let random_speed = sampler.sample_range_normal(5.0, 5.0);

    let random_velocity = sampler.sample_direction_uniform() * random_speed;

    // Random mesh transform.

    let random_transform = {
        let random_rotation = {
            let rotate_x = Quaternion::new(vec3::RIGHT, sampler.sample_range_uniform(0.0, TAU));
            let rotate_y = Quaternion::new(vec3::UP, sampler.sample_range_uniform(0.0, TAU));
            let rotate_z = Quaternion::new(vec3::FORWARD, sampler.sample_range_uniform(0.0, TAU));

            rotate_x * rotate_y * rotate_z
        };

        let random_translation = Mat4::translation(Vec3 {
            x: sampler.sample_range_normal(0.0, 25.0),
            y: sampler.sample_range_normal(25.0, 10.0),
            z: sampler.sample_range_normal(0.0, 25.0),
        });

        *random_rotation.mat() * random_translation
    };

    for point in &mut points {
        point.mass = PARTICLE_MASS;
        point.position = (Vec4::position(point.position) * random_transform).to_vec3();
        point.velocity = random_velocity;
    }

    // Random physics material.

    let material = {
        let random_dynamic_friction = sampler.sample_range_uniform(0.5, 0.9);
        let random_restitution = sampler.sample_range_uniform(0.5, 0.9);

        PhysicsMaterial {
            static_friction: 0.0,
            dynamic_friction: random_dynamic_friction,
            restitution: random_restitution,
        }
    };

    // Determines the set of triangles, based on the connectivity of struts with
    // points.

    let triangles = get_triangles(&points, &struts);

    let mut aabb = AABB::default();

    for point in &points {
        aabb.grow(&point.position);
    }

    aabb.recompute_derived_state();

    SpringyMesh {
        points,
        struts,
        material,
        state_index_offset: 0,
        triangles,
        aabb,
    }
}

fn get_triangles(points: &[Particle], struts: &[Strut]) -> Vec<Triangle> {
    let mut triangle_set = HashMap::<(usize, usize, usize), (usize, usize, usize)>::default();

    for strut in struts {
        if let Some(connected_points) = strut.edge.connected_points {
            // External strut connecting 2 points, and 2 "wings"; defines 2
            // triangles, which might each be shared with other external struts.

            // Collects index tuples of this edge's two adjacent triangles.

            let triangle_vertex_indices = [
                (strut.edge.points.0, strut.edge.points.1, connected_points.0),
                (connected_points.1, strut.edge.points.1, strut.edge.points.0),
            ];

            // Sorts triangle indices to avoid duplicates in the set.

            for (v0, v1, v2) in triangle_vertex_indices.into_iter() {
                let mut v = [v0, v1, v2];

                v.sort();

                let sorted_key = (v[0], v[1], v[2]);

                triangle_set.entry(sorted_key).or_insert((v2, v1, v0));
            }
        }
    }

    let triangles: Vec<Triangle> = triangle_set
        .into_iter()
        .map(|(_, (v0, v1, v2))| {
            Triangle::new(
                [v0, v1, v2],
                &points[v0].position,
                &points[v1].position,
                &points[v2].position,
            )
        })
        .collect();

    triangles
}
