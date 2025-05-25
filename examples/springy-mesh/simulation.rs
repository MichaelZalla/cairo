use std::{collections::HashMap, f32::consts::TAU};

use cairo::{
    animation::lerp,
    geometry::{
        intersect::{intersect_line_segment_plane, intersect_line_segment_triangle},
        primitives::line_segment::LineSegment,
    },
    matrix::Mat4,
    physics::simulation::{
        collision_response::{
            resolve_edge_edge_collision, resolve_point_plane_collision_approximate,
            resolve_vertex_face_collision,
        },
        force::{gravity::GRAVITY_POINT_FORCE, PointForce},
        state_vector::{FromStateVector, StateVector, ToStateVector},
    },
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::{
    integration::{integrate_midpoint_euler, system_dynamics_function},
    plane_collider::PlaneCollider,
    springy_mesh::{make_cube, make_springy_mesh, SpringyMesh},
};

pub const COMPONENTS_PER_PARTICLE: usize = 2; // { position, velocity }

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub struct PointFaceCollision {
    pub a_mesh_index: usize,
    pub a_point_index: usize,
    pub b_mesh_index: usize,
    pub b_face_index: usize,
    pub barycentric: Vec3,
    pub s: Vec3,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct EdgePair {
    pub a_mesh_index: usize,
    pub a_edge_index: usize,
    pub b_mesh_index: usize,
    pub b_edge_index: usize,
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub struct EdgeEdgeCollision {
    pub pair: EdgePair,
    pub s: f32,
    pub t: f32,
}

#[derive(Default, Debug, Clone)]
pub struct Simulation {
    pub forces: Vec<PointForce>,
    pub meshes: Vec<SpringyMesh>,
    pub static_plane_colliders: Vec<PlaneCollider>,
    pub vertex_collisions: Vec<PointFaceCollision>,
    pub closest_points: HashMap<EdgePair, Vec3>,
    pub edge_collisions: Vec<EdgeEdgeCollision>,
}

impl Simulation {
    pub fn tick(&mut self, h: f32, uptime_seconds: f32) {
        let num_points: usize = self.meshes.iter().map(|mesh| mesh.points.len()).sum();

        let mut state = StateVector::new(COMPONENTS_PER_PARTICLE, num_points);

        let n = state.len();

        // Copies current positions and velocities into the current state.

        let mut i = 0;

        for mesh in &mut self.meshes {
            mesh.state_index_offset = i;

            for point in &mesh.points {
                point.write_to(&mut state, n, i);

                i += 1;
            }
        }

        // Computes the derivative and integrate over h.

        let derivative =
            system_dynamics_function(&state, &self.forces, &mut self.meshes, uptime_seconds);

        let mut new_state = integrate_midpoint_euler(&state, &derivative, h);

        // Detects and resolves collisions with static colliders.

        self.check_static_collisions(&derivative, &state, &mut new_state, n, h);

        // Detects and resolves collisions between springy meshes.

        self.check_springy_mesh_collisions(&state, &mut new_state, n);

        // Copy new positions and velocities back the meshes' particles.
        // (Updates mesh AABB bounds and collision triangles).

        for mesh in &mut self.meshes {
            mesh.write_from(&new_state, n, mesh.state_index_offset);
        }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        // Render springy meshes.

        for mesh in &self.meshes {
            mesh.render(renderer);
        }

        // Visualize static plane colliders.
        for collider in &self.static_plane_colliders {
            collider.render(renderer);
        }
    }

    fn check_static_collisions(
        &self,
        derivative: &StateVector,
        state: &StateVector,
        new_state: &mut StateVector,
        n: usize,
        h: f32,
    ) {
        for mesh in &self.meshes {
            for i in 0..mesh.points.len() {
                let acceleration = derivative.data[mesh.state_index_offset + i + n];

                let start_position = state.data[mesh.state_index_offset + i];
                let mut end_position = new_state.data[mesh.state_index_offset + i];
                let mut end_velocity = new_state.data[mesh.state_index_offset + i + n];

                // We'll break early on the first collision (if any).

                for collider in &self.static_plane_colliders {
                    if let Some((f, intersection_point)) =
                        intersect_line_segment_plane(&collider.plane, start_position, end_position)
                    {
                        let penetration_depth = (end_position - intersection_point).mag();

                        {
                            // Subtracts any velocity accumulated while colliding.

                            let time_before_collision = h * f;
                            let time_after_collision = h - time_before_collision;

                            let accumulated_velocity = acceleration * 2.0 * time_after_collision;

                            end_velocity -= accumulated_velocity;
                        }

                        resolve_point_plane_collision_approximate(
                            collider.plane.normal,
                            &mesh.material,
                            &mut end_position,
                            &mut end_velocity,
                            penetration_depth,
                        );

                        new_state.data[mesh.state_index_offset + i + n] = end_velocity;
                        new_state.data[mesh.state_index_offset + i] = end_position;
                    }
                }
            }
        }
    }

    fn check_springy_mesh_collisions(
        &mut self,
        state: &StateVector,
        new_state: &mut StateVector,
        n: usize,
    ) {
        // Resets vertex-face and edge-edge collisions.

        self.vertex_collisions.clear();
        self.edge_collisions.clear();

        // Detect and resolve collisions between mesh pairs.

        for i in 0..self.meshes.len() {
            for j in i + 1..self.meshes.len() {
                // Checks for vertex-face collisions.

                for (a, b) in [(i, j), (j, i)] {
                    for p_i in 0..self.meshes[a].points.len() {
                        for tri_i in 0..self.meshes[b].triangles.len() {
                            // Checks mesh `a`, point at `p_i` against mesh `b`, face at `tri_i`.

                            if self.did_handle_point_face_collision(
                                a, p_i, b, tri_i, n, state, new_state,
                            ) {
                                println!("Handled vertex-triangle collision.");
                            }
                        }
                    }
                }

                // Checks for edge-edge collisions.

                for edge_i in 0..self.meshes[i].struts.len() {
                    // Skips internal struts.
                    if self.meshes[i].struts[edge_i]
                        .edge
                        .connected_points
                        .is_none()
                    {
                        continue;
                    }

                    for edge_j in 0..self.meshes[j].struts.len() {
                        // Skips internal struts.
                        if self.meshes[j].struts[edge_j]
                            .edge
                            .connected_points
                            .is_none()
                        {
                            continue;
                        }

                        // Checks mesh `i`, edge `edge_i` against mesh `j`, edge `tri_i`.

                        let pair = EdgePair {
                            a_mesh_index: i,
                            a_edge_index: edge_i,
                            b_mesh_index: j,
                            b_edge_index: edge_j,
                        };

                        if self.did_handle_edge_edge_collision(pair, n, new_state) {
                            println!("Handled edge-edge collision.");
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn did_handle_point_face_collision(
        &mut self,
        a: usize,
        p_i: usize,
        b: usize,
        tri_i: usize,
        n: usize,
        state: &StateVector,
        new_state: &mut StateVector,
    ) -> bool {
        // Constructs a line segment between the old and new vertex positions.

        let a_point_index = self.meshes[a].state_index_offset + p_i;

        let b_points_start_index = self.meshes[b].state_index_offset;

        let start_position = &state.data[a_point_index];
        let end_position = &new_state.data[a_point_index];

        let start_velocity = &state.data[a_point_index + n];
        let end_velocity = &new_state.data[a_point_index + n];

        let mut segment = LineSegment::new(*start_position, *end_position);

        // Updates the triangle (collider) associated with the face.

        // @NOTE The triangle is treated as a static collider, based on the new
        // positions of its three vertices in `new_state`.

        let triangle = &mut self.meshes[b].triangles[tri_i];

        let [i0, i1, i2] = triangle.vertices;

        let v0_position = &new_state.data[b_points_start_index + i0];
        let v1_position = &new_state.data[b_points_start_index + i1];
        let v2_position = &new_state.data[b_points_start_index + i2];

        triangle.update_vertex_positions(v0_position, v1_position, v2_position);

        let normal = triangle.plane.normal;

        // Performs a line-segment-triangle intersection test.

        match intersect_line_segment_triangle(&mut segment, triangle) {
            Some(barycentric) => {
                // Records the point-face collision.

                let s = segment.lerped();

                let collision = PointFaceCollision {
                    a_mesh_index: a,
                    a_point_index: p_i,
                    b_mesh_index: b,
                    b_face_index: tri_i,
                    barycentric,
                    s,
                };

                self.vertex_collisions.push(collision);

                // Uses the physics material associated with mesh A.

                let material = &self.meshes[a].material;

                // Gathers mass and velocity for collision body A (point).

                let point_mass = self.meshes[a].points[p_i].mass;

                let mut point_velocity = lerp(*start_velocity, *end_velocity, segment.t);

                // Gathers masses and velocities for collision body B (face).

                let (v0_mass, v1_mass, v2_mass) = {
                    (
                        self.meshes[b].points[i0].mass,
                        self.meshes[b].points[i1].mass,
                        self.meshes[b].points[i2].mass,
                    )
                };

                let mut v0_velocity = new_state.data[b_points_start_index + i0 + n];
                let mut v1_velocity = new_state.data[b_points_start_index + i1 + n];
                let mut v2_velocity = new_state.data[b_points_start_index + i2 + n];

                // Computes velocity updates for both bodies.

                resolve_vertex_face_collision(
                    material,
                    normal,
                    barycentric,
                    point_mass,
                    &mut point_velocity,
                    v0_mass,
                    &mut v0_velocity,
                    v1_mass,
                    &mut v1_velocity,
                    v2_mass,
                    &mut v2_velocity,
                );

                let penetration_depth = (s - end_position).dot(normal);

                let bias = if point_velocity.dot(normal) < 0.05 {
                    0.01
                } else {
                    0.0
                };

                let point_position = end_position
                    + normal * (penetration_depth * (1.0 + material.restitution) + bias);

                // Updates point position and velocity.
                new_state.data[a_point_index] = point_position;
                new_state.data[a_point_index + n] = point_velocity;

                // Updates face vertex positions and velocities.
                new_state.data[b_points_start_index + n + i0] = v0_velocity;
                new_state.data[b_points_start_index + n + i1] = v1_velocity;
                new_state.data[b_points_start_index + n + i2] = v2_velocity;

                true
            }
            None => false,
        }
    }

    fn did_handle_edge_edge_collision(
        &mut self,
        pair: EdgePair,
        n: usize,
        new_state: &mut StateVector,
    ) -> bool {
        // Extracts the edge's vertex positions.

        let (p1, p2) = {
            let mesh_a = &self.meshes[pair.a_mesh_index];
            let edge_a = &mesh_a.struts[pair.a_edge_index].edge;

            (
                new_state.data[mesh_a.state_index_offset + edge_a.points.0],
                new_state.data[mesh_a.state_index_offset + edge_a.points.1],
            )
        };

        let (q1, q2) = {
            let mesh_b = &self.meshes[pair.b_mesh_index];
            let edge_b = &mesh_b.struts[pair.b_edge_index].edge;

            (
                new_state.data[mesh_b.state_index_offset + edge_b.points.0],
                new_state.data[mesh_b.state_index_offset + edge_b.points.1],
            )
        };

        // A vector running the length of edge A.
        let a = p2 - p1;

        // A vector running the length of edge B.
        let b = q2 - q1;

        // The direction of the vector between closest points.
        let normal = a.cross(b).as_normal();

        // Edges A and B can be expressed parametrically, with parameters `s` and `t`:
        //
        //   A(s) = p_1 + (p_2 - p_1) * s
        //   B(t) = q_1 + (q_2 - q_1) * t
        //
        // We want to compute an `s` and a `t` such that, when plugged in to the
        // parametric expressions above, A(s) and B(t) are the closest points
        // between the edges.

        let r = q1 - p1;

        let a_norm_cross_n = a.as_normal().cross(normal);
        let b_norm_cross_n = b.as_normal().cross(normal);

        let s = r.dot(b_norm_cross_n) / a.dot(b_norm_cross_n);
        let t = -r.dot(a_norm_cross_n) / b.dot(a_norm_cross_n);

        // If s < 0 or s > 1, then the closest point on the line forming A falls
        // outside of A.

        if !(0.0..=1.0).contains(&s) {
            return false;
        }

        // If t < 0 or s > 1, then the closest point on the line forming B falls
        // outside of B.

        if !(0.0..=1.0).contains(&t) {
            return false;
        }

        let p_a = p1 + (p2 - p1) * s;
        let q_a = q1 + (q2 - q1) * t;

        let m: Vec3 = q_a - p_a;

        if let Some(prev_m) = self.closest_points.get_mut(&pair) {
            // Compares the dot products of current and previous `m` vectors.

            if m.dot(*prev_m) < 0.0 {
                // Updates prev_m to m.

                *prev_m = m;

                let collision = EdgeEdgeCollision {
                    pair: pair.clone(),
                    s,
                    t,
                };

                // Resolves the edge-edge collision.

                // Uses the physics material associated with mesh A.

                let material = &self.meshes[pair.a_mesh_index].material;

                // Gathers masses and velocities for edge A's vertices.

                let (p1_mass, mut p1_velocity, p2_mass, mut p2_velocity) = {
                    let mesh_a = &self.meshes[pair.a_mesh_index];
                    let edge_a = &mesh_a.struts[pair.a_edge_index].edge;

                    (
                        mesh_a.points[edge_a.points.0].mass,
                        new_state.data[mesh_a.state_index_offset + edge_a.points.0 + n],
                        mesh_a.points[edge_a.points.1].mass,
                        new_state.data[mesh_a.state_index_offset + edge_a.points.1 + n],
                    )
                };

                // Gathers masses and velocities for edge B's vertices.

                let (q1_mass, mut q1_velocity, q2_mass, mut q2_velocity) = {
                    let mesh_b = &self.meshes[pair.b_mesh_index];
                    let edge_b = &mesh_b.struts[pair.b_edge_index].edge;

                    (
                        mesh_b.points[edge_b.points.0].mass,
                        new_state.data[mesh_b.state_index_offset + edge_b.points.0 + n],
                        mesh_b.points[edge_b.points.1].mass,
                        new_state.data[mesh_b.state_index_offset + edge_b.points.1 + n],
                    )
                };

                // Computes velocity updates for collision response.

                resolve_edge_edge_collision(
                    material,
                    p1_mass,
                    &mut p1_velocity,
                    p2_mass,
                    &mut p2_velocity,
                    q1_mass,
                    &mut q1_velocity,
                    q2_mass,
                    &mut q2_velocity,
                    s,
                    t,
                    m.as_normal(),
                );

                {
                    let mesh_a = &self.meshes[pair.a_mesh_index];
                    let edge_a = &mesh_a.struts[pair.a_edge_index].edge;

                    new_state.data[mesh_a.state_index_offset + edge_a.points.0 + n] = p1_velocity;
                    new_state.data[mesh_a.state_index_offset + edge_a.points.1 + n] = p2_velocity;
                }

                {
                    let mesh_b = &self.meshes[pair.b_mesh_index];
                    let edge_b = &mesh_b.struts[pair.b_edge_index].edge;

                    new_state.data[mesh_b.state_index_offset + edge_b.points.0 + n] = q1_velocity;
                    new_state.data[mesh_b.state_index_offset + edge_b.points.1 + n] = q2_velocity;
                }

                self.edge_collisions.push(collision);

                return true;
            }
        }

        self.closest_points.insert(pair, m);

        false
    }
}

pub fn make_simulation(sampler: &mut RandomSampler<1024>) -> Simulation {
    // Forces.

    let forces: Vec<PointForce> = vec![GRAVITY_POINT_FORCE];

    // Springy meshes.

    static NUM_MESHES: usize = 100;
    static SIDE_LENGTH: f32 = 3.0;

    let mut meshes = Vec::with_capacity(NUM_MESHES);

    for _ in 0..meshes.capacity() {
        let (points, struts) = make_cube(SIDE_LENGTH);

        let mut mesh = make_springy_mesh(points, struts, sampler);

        let random_speed = sampler.sample_range_normal(5.0, 5.0);

        let random_velocity = sampler.sample_direction_uniform() * random_speed;

        // Random mesh transform.

        let random_transform = {
            let random_rotation = {
                let rotate_x = Quaternion::new(vec3::RIGHT, sampler.sample_range_uniform(0.0, TAU));
                let rotate_y = Quaternion::new(vec3::UP, sampler.sample_range_uniform(0.0, TAU));
                let rotate_z =
                    Quaternion::new(vec3::FORWARD, sampler.sample_range_uniform(0.0, TAU));

                rotate_x * rotate_y * rotate_z
            };

            let random_translation = Mat4::translation(Vec3 {
                x: sampler.sample_range_normal(0.0, 25.0),
                y: sampler.sample_range_normal(25.0, 10.0),
                z: sampler.sample_range_normal(0.0, 25.0),
            });

            *random_rotation.mat() * random_translation
        };

        for point in &mut mesh.points {
            point.position = (Vec4::position(point.position) * random_transform).to_vec3();
            point.velocity = random_velocity;
        }

        mesh.update_aabb();
        mesh.update_triangles();

        meshes.push(mesh);
    }

    // Ground collider plane.

    let static_plane_colliders = vec![PlaneCollider::new(Default::default(), vec3::UP)];

    Simulation {
        meshes,
        forces,
        static_plane_colliders,
        ..Default::default()
    }
}
