use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
};

use bitflags::bitflags;

use cairo::{
    animation::lerp,
    color,
    geometry::primitives::{aabb::AABB, triangle::Triangle},
    matrix::Mat4,
    physics::{
        material::PhysicsMaterial,
        simulation::{
            particle::Particle,
            state_vector::{FromStateVector, StateVector},
        },
    },
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    render::Renderer,
    scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::strut::{DAMPING_RATIO, Edge, PARTICLE_MASS, Strut, UNDAMPED_PERIOD};

#[allow(dead_code)]
pub enum SpringyMeshType {
    Spring { with_connected_points: bool },
    Tetrahedron,
    Cube,
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct SpringyMeshDebugFlags: u32 {
        const DRAW_AABB = 1;
        const DRAW_AABB_BOUNDING_SPHERE = 1 << 1;
        const DRAW_POINTS = 1 << 2;
        const DRAW_POINT_VELOCITIES = 1 << 3;
        const DRAW_POINT_COLLISIONS = 1 << 4;
        const DRAW_STRUTS = 1 << 5;
        const DRAW_STRUT_EDGE_COLLISIONS = 1 << 6;
        const DRAW_STRUT_SPRING_ACCELERATIONS = 1 << 7;
        const DRAW_TORSIONAL_STRUT_FACE_NORMALS = 1 << 8;
        const DRAW_TORSIONAL_STRUT_R_VECTORS = 1 << 9;
        const DRAW_TORSIONAL_STRUT_ROTATIONAL_FORCES = 1 << 10;
        const DRAW_FACE_COLLISIONS = 1 << 11;
    }
}

impl Default for SpringyMeshDebugFlags {
    fn default() -> Self {
        SpringyMeshDebugFlags::DRAW_POINTS | SpringyMeshDebugFlags::DRAW_STRUTS
    }
}

#[derive(Default, Debug, Clone)]
pub struct SpringyMesh {
    pub material: PhysicsMaterial,
    pub points: Vec<Particle>,
    pub struts: Vec<Strut>,
    pub state_index_offset: usize,
    pub triangles: Vec<Triangle>,
    pub aabb: AABB,
    pub debug_flags: SpringyMeshDebugFlags,
}

impl FromStateVector for SpringyMesh {
    fn write_from(&mut self, state: &StateVector, n: usize, _i: usize) {
        for (i, point) in &mut self.points.iter_mut().enumerate() {
            point.write_from(state, n, self.state_index_offset + i);
        }

        self.update_triangles();
    }
}

impl SpringyMesh {
    pub fn update_aabb(&mut self) {
        self.aabb = AABB::default();

        for point in &self.points {
            self.aabb.grow(&point.position);
        }

        self.aabb.recompute_derived_state();
    }

    pub fn update_triangles(&mut self) {
        for triangle in &mut self.triangles {
            let (v0, v1, v2) = (
                &self.points[triangle.vertices[0]].position,
                &self.points[triangle.vertices[1]].position,
                &self.points[triangle.vertices[2]].position,
            );

            triangle.update_vertex_positions(v0, v1, v2);
        }
    }

    pub fn reset_collisions(&mut self) {
        for point in &mut self.points {
            point.did_collide = false;
        }

        for strut in &mut self.struts {
            strut.edge.did_collide = false;
        }

        for tri in &mut self.triangles {
            tri.collision_point.take();
        }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        if self.debug_flags.contains(SpringyMeshDebugFlags::DRAW_AABB) {
            // Visualizes the mesh's AABB.

            renderer.render_aabb(&self.aabb, Default::default(), color::DARK_GRAY);
        }

        if self
            .debug_flags
            .contains(SpringyMeshDebugFlags::DRAW_AABB_BOUNDING_SPHERE)
        {
            // Visualizes the mesh's AABB bounding sphere.

            let transform = {
                let scale = Mat4::scale_uniform(self.aabb.bounding_sphere_radius);
                let translate = Mat4::translation(self.aabb.center());

                scale * translate
            };

            renderer.render_empty(
                &transform,
                EmptyDisplayKind::Sphere(16),
                false,
                Some(color::LIGHT_GRAY),
            );
        }

        if self.debug_flags.intersects(
            SpringyMeshDebugFlags::DRAW_POINTS | SpringyMeshDebugFlags::DRAW_POINT_VELOCITIES,
        ) {
            for point in &self.points {
                if self
                    .debug_flags
                    .contains(SpringyMeshDebugFlags::DRAW_POINTS)
                {
                    // Visualizes the point.

                    let transform =
                        Mat4::scale(vec3::ONES * 0.1) * Mat4::translation(point.position);

                    let color = if point.did_collide
                        && self
                            .debug_flags
                            .contains(SpringyMeshDebugFlags::DRAW_POINT_COLLISIONS)
                    {
                        color::RED
                    } else {
                        color::ORANGE
                    };

                    renderer.render_empty(
                        &transform,
                        EmptyDisplayKind::Sphere(12),
                        false,
                        Some(color),
                    );
                }

                if self
                    .debug_flags
                    .contains(SpringyMeshDebugFlags::DRAW_POINT_VELOCITIES)
                {
                    // Visualizes the point's velocity.

                    renderer.render_line(
                        point.position,
                        point.position + point.velocity,
                        color::YELLOW,
                    );
                }
            }
        }

        if self.debug_flags.intersects(
            SpringyMeshDebugFlags::DRAW_STRUTS
                | SpringyMeshDebugFlags::DRAW_STRUT_EDGE_COLLISIONS
                | SpringyMeshDebugFlags::DRAW_STRUT_SPRING_ACCELERATIONS
                | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_FACE_NORMALS
                | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_R_VECTORS
                | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_ROTATIONAL_FORCES
                | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_FACE_NORMALS
                | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_ROTATIONAL_FORCES,
        ) {
            for strut in &self.struts {
                let start = self.points[strut.edge.points.0].position;
                let end = self.points[strut.edge.points.1].position;

                if self
                    .debug_flags
                    .contains(SpringyMeshDebugFlags::DRAW_STRUTS)
                {
                    // Visualizes the strut.

                    let color = if strut.edge.did_collide
                        && self
                            .debug_flags
                            .contains(SpringyMeshDebugFlags::DRAW_STRUT_EDGE_COLLISIONS)
                    {
                        color::RED
                    } else {
                        strut.edge.color
                    };

                    renderer.render_line(start, end, color);
                }

                if self
                    .debug_flags
                    .contains(SpringyMeshDebugFlags::DRAW_STRUT_SPRING_ACCELERATIONS)
                {
                    // Visualizes the strut's spring accelerations.

                    renderer.render_line(
                        start,
                        start + strut.spring_acceleration / 10.0,
                        color::BLUE,
                    );
                    renderer.render_line(end, end - strut.spring_acceleration / 10.0, color::RED);
                }

                // Visualizes the strut's torsional spring forces.

                if let (Some(connected_points), true) = (
                    &strut.edge.connected_points,
                    self.debug_flags.intersects(
                        SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_FACE_NORMALS
                            | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_R_VECTORS
                            | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_ROTATIONAL_FORCES,
                    ),
                ) {
                    let h = (end - start).as_normal();

                    let left = self.points[connected_points.0].position;
                    let right = self.points[connected_points.1].position;

                    if self.debug_flags.intersects(
                        SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_FACE_NORMALS
                            | SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_R_VECTORS,
                    ) {
                        let normals =
                            Strut::get_surface_normals_edge_points(&strut.edge, &self.points)
                                .unwrap();

                        for i in 0..=1 {
                            let c = if i == 0 { &left } else { &right };

                            let start_c = *c - start;

                            let r_c = start_c - h * start_c.dot(h);

                            if self
                                .debug_flags
                                .contains(SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_R_VECTORS)
                            {
                                // Visualizes r-vector (from h to left point or right point).

                                renderer.render_line(*c - r_c, *c, color::YELLOW);
                                renderer.render_line(*c - r_c, start, color::WHITE);
                            }

                            if self
                                .debug_flags
                                .contains(SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_FACE_NORMALS)
                            {
                                // Visualize normals of the faces formed by
                                // start, end, and the connected point.

                                let midpoint_c = lerp(*c, *c - r_c, 0.5);

                                let normal_c = if i == 0 { normals.0 } else { normals.1 };

                                renderer.render_line(
                                    midpoint_c,
                                    midpoint_c + normal_c,
                                    color::GREEN,
                                );
                            }
                        }
                    }

                    if self
                        .debug_flags
                        .contains(SpringyMeshDebugFlags::DRAW_TORSIONAL_STRUT_ROTATIONAL_FORCES)
                    {
                        // Visualize the rotational forces applied to start, end, left, and right.

                        for (i, p) in [&start, &end, &left, &right].iter().enumerate() {
                            renderer.render_line(
                                **p,
                                **p + (strut.rotational_forces[i]),
                                color::ORANGE,
                            );
                        }
                    }
                }
            }
        }

        // Visualize face collisions.

        if self
            .debug_flags
            .contains(SpringyMeshDebugFlags::DRAW_FACE_COLLISIONS)
        {
            for tri in &self.triangles {
                if let Some(barycentric) = &tri.collision_point {
                    let vertex_indices = (tri.vertices[0], tri.vertices[1], tri.vertices[2]);

                    let vertices = (
                        &self.points[vertex_indices.0].position,
                        &self.points[vertex_indices.1].position,
                        &self.points[vertex_indices.2].position,
                    );

                    let collision_point = *vertices.0 * barycentric.x
                        + *vertices.1 * barycentric.y
                        + *vertices.2 * barycentric.z;

                    renderer.render_line(*vertices.0, collision_point, color::RED);
                    renderer.render_line(*vertices.1, collision_point, color::RED);
                    renderer.render_line(*vertices.2, collision_point, color::RED);
                }
            }
        }
    }
}

#[allow(unused)]
pub fn make_spring(side_length: f32, with_connected_points: bool) -> (Vec<Particle>, Vec<Strut>) {
    let factor = side_length / 2.0;

    let mut vertices = vec![vec3::FORWARD * factor, -vec3::FORWARD * factor];

    if with_connected_points {
        vertices.push(vec3::RIGHT * factor + vec3::UP * factor);
        vertices.push(-vec3::RIGHT * factor + vec3::UP * factor);
    }

    // Connect points with edges.

    let mut edge_data = if with_connected_points {
        vec![
            (0, 1, 3, 2, color::LIGHT_GRAY),
            (0, 2, usize::MAX, usize::MAX, color::LIGHT_GRAY),
            (2, 1, usize::MAX, usize::MAX, color::LIGHT_GRAY),
            (0, 3, usize::MAX, usize::MAX, color::LIGHT_GRAY),
            (3, 1, usize::MAX, usize::MAX, color::LIGHT_GRAY),
        ]
    } else {
        vec![(0, 1, usize::MAX, usize::MAX, color::LIGHT_GRAY)]
    };

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
            did_collide: false,
        })
        .collect();

    make_points_and_struts(vertices, edges)
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
            did_collide: false,
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
        (0, 1, 4, 2, color::LIGHT_GRAY),
        (1, 2, 6, 0, color::LIGHT_GRAY),
        (2, 3, 6, 0, color::LIGHT_GRAY),
        (3, 0, 4, 2, color::LIGHT_GRAY),
        // Back loop (4)
        (4, 5, 7, 1, color::LIGHT_GRAY),
        (5, 6, 7, 1, color::LIGHT_GRAY),
        (6, 7, 5, 3, color::LIGHT_GRAY),
        (7, 4, 5, 3, color::LIGHT_GRAY),
        // Front-to-back connections (4)
        (0, 4, 3, 1, color::LIGHT_GRAY),
        (1, 5, 4, 6, color::LIGHT_GRAY),
        (2, 6, 1, 3, color::LIGHT_GRAY),
        (3, 7, 6, 4, color::LIGHT_GRAY),
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
            did_collide: false,
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
    for point in &mut points {
        point.mass = PARTICLE_MASS;
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
        triangles,
        aabb,
        ..Default::default()
    }
}

pub fn make_springy_meshes(
    count: usize,
    mesh_type: SpringyMeshType,
    side_length: f32,
    scale: f32,
    sampler: &mut RandomSampler<1024>,
) -> Vec<SpringyMesh> {
    let mut meshes = Vec::with_capacity(count);

    for _ in 0..count {
        let (points, struts) = match mesh_type {
            SpringyMeshType::Spring {
                with_connected_points,
            } => make_spring(side_length, with_connected_points),
            SpringyMeshType::Tetrahedron => make_tetrahedron(side_length),
            SpringyMeshType::Cube => make_cube(side_length),
        };

        let mut mesh = make_springy_mesh(points, struts, sampler);

        let speed = sampler.sample_range_normal(5.0, 5.0);

        let velocity = sampler.sample_direction_uniform() * speed;

        // Mesh transform.

        let transform = {
            let rotation = {
                let rotate_x = Quaternion::new(vec3::RIGHT, sampler.sample_range_uniform(0.0, TAU));
                let rotate_y = Quaternion::new(vec3::UP, sampler.sample_range_uniform(0.0, TAU));
                let rotate_z =
                    Quaternion::new(vec3::FORWARD, sampler.sample_range_uniform(0.0, TAU));

                rotate_x * rotate_y * rotate_z
            };

            static BOUNDS: f32 = 15.0;

            let translation = Mat4::translation(Vec3 {
                x: sampler.sample_range_normal(0.0, BOUNDS),
                y: sampler.sample_range_normal(35.0, 15.0),
                z: sampler.sample_range_normal(0.0, BOUNDS),
            });

            let scale_transform = Mat4::scale_uniform(scale);

            scale_transform * *rotation.mat() * translation
        };

        for point in &mut mesh.points {
            point.position = (Vec4::position(point.position) * transform).to_vec3();
            point.velocity = velocity;
        }

        for strut in &mut mesh.struts {
            // P_n = 2 Pi sqrt(m / k)
            // P_n^2 = 4 Pi^2 (m/k)
            // k P_n^2 = 4 Pi^2 m
            // k = (4 Pi^2 m) / P_n^2

            let k = (4.0 * PI * PI * PARTICLE_MASS) / (UNDAMPED_PERIOD * UNDAMPED_PERIOD);

            // z = c / 2 sqrt(m * k)
            // c = z * 2 * sqrt(mk)
            // c^2 = z^2 * 2^2 * mk
            // c^2 = z^2 * 2^2 * mk
            // c = z 2 sqrt(mk)

            let c = DAMPING_RATIO * 2.0 * (PARTICLE_MASS * k).sqrt();

            strut.spring_strength = k / strut.rest_length;
            strut.spring_damper = c / strut.rest_length;

            if strut.edge.connected_points.is_some() {
                // P_n = 2 Pi sqrt(m r^2 / k)
                // P_n = 2 Pi r sqrt(m / k)
                // P_n^2 = 4 Pi^2 r^2 (m/k)
                // k P_n^2 = 4 Pi^2 m r^2
                // k = (4 Pi^2 m r^2) / P_n^2

                let factor = side_length / 2.0;

                let r = (factor * factor + factor * factor).sqrt();

                let k_a =
                    (4.0 * PI * PI * PARTICLE_MASS * r * r) / (UNDAMPED_PERIOD * UNDAMPED_PERIOD);

                // z = c / 2 sqrt(m r^2 k)
                // c = z 2 r sqrt(mk)
                // c^2 = z^2 2^2 r^2 mk
                // c^2 = z^2 2^2 r^2 mk
                // c = z 2 r sqrt(mk)

                let c_a = DAMPING_RATIO * 2.0 * r * (PARTICLE_MASS * k_a).sqrt();

                strut.torsional_strength = k_a;
                strut.torsional_damper = c_a;
            }
        }

        mesh.update_aabb();
        mesh.update_triangles();

        meshes.push(mesh);
    }

    meshes
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
