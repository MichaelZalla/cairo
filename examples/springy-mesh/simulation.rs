use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use cairo::{
    animation::lerp,
    geometry::{
        intersect::{intersect_line_segment_plane, intersect_line_segment_triangle},
        primitives::line_segment::{get_closest_points_between_segments, LineSegment},
    },
    matrix::Mat4,
    physics::simulation::{
        collision_response::{
            resolve_edge_edge_collision, resolve_point_plane_collision_approximate,
            resolve_vertex_face_collision,
        },
        force::PointForce,
        state_vector::{FromStateVector, StateVector, ToStateVector},
    },
    random::sampler::RandomSampler,
    software_renderer::SoftwareRenderer,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::{
    integration::{integrate_midpoint_euler, system_dynamics_function},
    plane_collider::PlaneCollider,
    springy_mesh::{make_spring, make_springy_mesh, SpringyMesh},
    strut::{DAMPING_RATIO, PARTICLE_MASS, UNDAMPED_PERIOD},
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

        static USE_HIGHER_ORDER_INTEGRATOR: bool = true;

        let (derivative, mut new_state) = if USE_HIGHER_ORDER_INTEGRATOR {
            let derivative: StateVector = {
                let s1 = state.clone();

                let k1 = system_dynamics_function(
                    &s1.clone(),
                    &self.forces,
                    &mut self.meshes,
                    uptime_seconds,
                );

                let s2 = s1.clone() + k1.clone() * (h * 0.5);

                let k2 =
                    system_dynamics_function(&s2, &self.forces, &mut self.meshes, uptime_seconds);

                let s3 = s2 + k2.clone() * (h * 0.5);

                let k3 =
                    system_dynamics_function(&s3, &self.forces, &mut self.meshes, uptime_seconds);

                let s4 = s3 + k3.clone() * h;

                let k4 =
                    system_dynamics_function(&s4, &self.forces, &mut self.meshes, uptime_seconds);

                (k1 + k2 * 2.0 + k3 * 2.0 + k4) * 0.166_666_67
            };

            let new_state = state.clone() + derivative.clone() * h;

            (derivative, new_state)
        } else {
            let derivative =
                system_dynamics_function(&state, &self.forces, &mut self.meshes, uptime_seconds);

            let new_state = integrate_midpoint_euler(&state, &derivative, h);

            (derivative, new_state)
        };

        // Resets mesh collision data for visual debugging.

        for mesh in &mut self.meshes {
            mesh.reset_collisions();
        }

        // Detects and resolves collisions with static colliders.

        self.check_static_collisions(&derivative, &state, &mut new_state, n, h);

        // Detects and resolves collisions between springy meshes.

        self.check_springy_mesh_collisions(&derivative, &state, &mut new_state, n, h);

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
                        {
                            // Subtracts any velocity accumulated while colliding.

                            let time_before_collision = h * f;
                            let time_after_collision = h - time_before_collision;

                            let accumulated_velocity = acceleration * 2.0 * time_after_collision;

                            end_velocity -= accumulated_velocity;
                        }

                        let penetration_depth = (end_position - intersection_point).mag();

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
        derivative: &StateVector,
        state: &StateVector,
        new_state: &mut StateVector,
        n: usize,
        h: f32,
    ) {
        // Resets vertex-face and edge-edge collisions.

        self.vertex_collisions.clear();
        self.edge_collisions.clear();

        let mut edge_pairs_tested = HashSet::<EdgePair>::default();

        // Detect and resolve collisions between mesh pairs.

        for i in 0..self.meshes.len() {
            for j in i + 1..self.meshes.len() {
                self.check_springy_mesh_pair(
                    derivative,
                    state,
                    new_state,
                    n,
                    h,
                    &mut edge_pairs_tested,
                    i,
                    j,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_springy_mesh_pair(
        &mut self,
        derivative: &StateVector,
        state: &StateVector,
        new_state: &mut StateVector,
        n: usize,
        h: f32,
        edge_pairs_tested: &mut HashSet<EdgePair>,
        i: usize,
        j: usize,
    ) {
        // Checks for vertex-face collisions.

        for (a, b) in [(i, j), (j, i)] {
            for p_i in 0..self.meshes[a].points.len() {
                for tri_i in 0..self.meshes[b].triangles.len() {
                    // Checks mesh `a`, point at `p_i` against mesh `b`, face at `tri_i`.

                    if let Some(barycentric) = self.did_handle_point_face_collision(
                        a, p_i, b, tri_i, n, derivative, state, new_state, h,
                    ) {
                        self.meshes[a].points[p_i].did_collide = true;

                        self.meshes[b].triangles[tri_i]
                            .collision_point
                            .replace(barycentric);
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

                // Avoids testing edge pair (j,i) after testing edge pair (i,j).

                if !edge_pairs_tested.contains(&pair) {
                    edge_pairs_tested.insert(pair.clone());

                    if self.did_handle_edge_edge_collision(pair, n, derivative, state, new_state, h)
                    {
                        self.meshes[i].struts[edge_i].edge.did_collide = true;
                        self.meshes[j].struts[edge_j].edge.did_collide = true;
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
        derivative: &StateVector,
        state: &StateVector,
        new_state: &mut StateVector,
        h: f32,
    ) -> Option<Vec3> {
        // Constructs a line segment between the old and new vertex positions.

        let a_point_index = self.meshes[a].state_index_offset + p_i;

        let b_points_start_index = self.meshes[b].state_index_offset;

        let start_position = state.data[a_point_index];
        let end_position = new_state.data[a_point_index];

        let mut segment = LineSegment::new(start_position, end_position);

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
                let time_before_collision = h * segment.t;
                let time_after_collision = h - time_before_collision;

                // Subtracts any velocity accumulated by the vertex while colliding.

                {
                    let acceleration = derivative.data[a_point_index + n];

                    let accumulated_velocity = acceleration * 2.0 * time_after_collision;

                    new_state.data[a_point_index + n] -= accumulated_velocity;
                }

                // Subtracts any velocity accumulated by the triangle vertices while colliding.

                for i in [i0, i1, i2] {
                    let acceleration = derivative.data[b_points_start_index + i + n];

                    let accumulated_velocity = acceleration * 2.0 * time_after_collision;

                    new_state.data[b_points_start_index + i + n] -= accumulated_velocity;
                }

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

                let start_velocity = &state.data[a_point_index + n];
                let end_velocity = &new_state.data[a_point_index + n];

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

                // Updates point velocity.

                new_state.data[a_point_index + n] = point_velocity;

                // Updates point position.

                new_state.data[a_point_index] = {
                    let velocity_in = new_state.data[a_point_index + n];

                    let position_at_collision =
                        state.data[a_point_index] + velocity_in * time_before_collision;

                    let velocity_out = new_state.data[a_point_index + n];

                    let bias = if velocity_in.dot(normal) < 0.05 {
                        0.01
                    } else {
                        0.0
                    };

                    position_at_collision + velocity_out * time_after_collision + bias
                };

                // Updates face vertex velocities.

                new_state.data[b_points_start_index + i0 + n] = v0_velocity;
                new_state.data[b_points_start_index + i1 + n] = v1_velocity;
                new_state.data[b_points_start_index + i2 + n] = v2_velocity;

                // Updates face vertex positions.

                for (i, velocity_out) in &[(i0, v0_velocity), (i1, v1_velocity), (i2, v2_velocity)]
                {
                    let velocity_in = new_state.data[b_points_start_index + i + n];

                    let position_at_collision =
                        state.data[b_points_start_index + i] + velocity_in * time_before_collision;

                    let bias = if velocity_in.dot(-normal) < 0.05 {
                        0.01
                    } else {
                        0.0
                    };

                    new_state.data[b_points_start_index + i] =
                        position_at_collision + *velocity_out * time_after_collision + bias;
                }

                Some(barycentric)
            }
            None => None,
        }
    }

    fn did_handle_edge_edge_collision(
        &mut self,
        pair: EdgePair,
        n: usize,
        derivative: &StateVector,
        state: &StateVector,
        new_state: &mut StateVector,
        h: f32,
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

        match get_closest_points_between_segments(p1, p2, q1, q2) {
            Some(((p_a, u), (q_a, v))) => {
                let m: Vec3 = q_a - p_a;

                // Compares the dot products of current and previous `m` vectors.

                if self.closest_points.contains_key(&pair)
                    && m.dot(*self.closest_points.get(&pair).unwrap()) < 0.0
                {
                    let collision = EdgeEdgeCollision {
                        pair: pair.clone(),
                        s: u,
                        t: v,
                    };

                    let prev_m = *self.closest_points.get(&pair).unwrap();

                    let t = {
                        let prev_m_mag = prev_m.mag();
                        let m_mag = m.mag();

                        prev_m_mag / (prev_m_mag + m_mag)
                    };

                    let time_before_collision = h * t;
                    let time_after_collision = h - time_before_collision;

                    // Resolves the edge-edge collision.

                    // Uses the physics material associated with mesh A.

                    let material = &self.meshes[pair.a_mesh_index].material;

                    // Gathers masses and velocities for edge A's vertices.

                    let (p1_mass, mut p1_velocity, p2_mass, mut p2_velocity) = {
                        let mesh_a = &self.meshes[pair.a_mesh_index];
                        let edge_a = &mesh_a.struts[pair.a_edge_index].edge;

                        let p1_mass = mesh_a.points[edge_a.points.0].mass;
                        let p2_mass = mesh_a.points[edge_a.points.1].mass;

                        let p1_index = mesh_a.state_index_offset + edge_a.points.0;
                        let p2_index = mesh_a.state_index_offset + edge_a.points.1;

                        // Subtracts any velocity accumulated while colliding.

                        let p1_velocity = new_state.data[p1_index + n]
                            - derivative.data[p1_index + n] * time_after_collision;

                        let p2_velocity = new_state.data[p2_index + n]
                            - derivative.data[p2_index + n] * time_after_collision;

                        (p1_mass, p1_velocity, p2_mass, p2_velocity)
                    };

                    // Gathers masses and velocities for edge B's vertices.

                    let (q1_mass, mut q1_velocity, q2_mass, mut q2_velocity) = {
                        let mesh_b = &self.meshes[pair.b_mesh_index];
                        let edge_b = &mesh_b.struts[pair.b_edge_index].edge;

                        let q1_mass = mesh_b.points[edge_b.points.0].mass;
                        let q2_mass = mesh_b.points[edge_b.points.1].mass;

                        let q1_index = mesh_b.state_index_offset + edge_b.points.0;
                        let q2_index = mesh_b.state_index_offset + edge_b.points.1;

                        // Subtracts any velocity accumulated while colliding.

                        let q1_velocity = new_state.data[q1_index + n]
                            - derivative.data[q1_index + n] * time_after_collision;

                        let q2_velocity = new_state.data[q2_index + n]
                            - derivative.data[q2_index + n] * time_after_collision;

                        (q1_mass, q1_velocity, q2_mass, q2_velocity)
                    };

                    // Computes velocity updates for collision response.

                    let normal = m.as_normal();

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
                        u,
                        v,
                        normal,
                    );

                    let mesh_a = &self.meshes[pair.a_mesh_index];
                    let edge_a = &mesh_a.struts[pair.a_edge_index].edge;

                    let mesh_b = &self.meshes[pair.b_mesh_index];
                    let edge_b = &mesh_b.struts[pair.b_edge_index].edge;

                    let (p1, p2, q1, q2) = (
                        mesh_a.state_index_offset + edge_a.points.0,
                        mesh_a.state_index_offset + edge_a.points.1,
                        mesh_b.state_index_offset + edge_b.points.0,
                        mesh_b.state_index_offset + edge_b.points.1,
                    );

                    // Updates edge vertex velocities.

                    new_state.data[p1 + n] = p1_velocity;
                    new_state.data[p2 + n] = p2_velocity;
                    new_state.data[q1 + n] = q1_velocity;
                    new_state.data[q2 + n] = q2_velocity;

                    // Updates edge vertex positions.

                    let bias = normal * 0.01;

                    new_state.data[p1] = state.data[p1]
                        + derivative.data[p1] * time_before_collision
                        + p1_velocity * time_after_collision
                        + bias;

                    new_state.data[p2] = state.data[p2]
                        + derivative.data[p2] * time_before_collision
                        + p2_velocity * time_after_collision
                        + bias;

                    new_state.data[q1] = state.data[q1]
                        + derivative.data[q1] * time_before_collision
                        + q1_velocity * time_after_collision
                        - bias;

                    new_state.data[q2] = state.data[q2]
                        + derivative.data[q2] * time_before_collision
                        + q2_velocity * time_after_collision
                        - bias;

                    self.edge_collisions.push(collision);

                    true
                } else {
                    self.closest_points.insert(pair, m);

                    false
                }
            }
            None => {
                self.closest_points.remove(&pair);

                false
            }
        }
    }
}

pub fn make_simulation(sampler: &mut RandomSampler<1024>) -> Simulation {
    // Forces.

    let forces: Vec<PointForce> = vec![
        // Gravity
        // GRAVITY_POINT_FORCE,
    ];

    // Springy meshes.

    static NUM_MESHES: usize = 4;

    let mut meshes = Vec::with_capacity(NUM_MESHES);

    for i in 0..meshes.capacity() {
        let side_length = 4.0;

        let (points, struts) = make_spring(side_length, true);

        let mut mesh = make_springy_mesh(points, struts, sampler);

        // Mesh transform.

        let transform = {
            let translation = Mat4::translation(Vec3 {
                x: -(NUM_MESHES as f32 * side_length * 2.0) / 2.0 + i as f32 * side_length * 2.0,
                y: 1.0,
                ..Default::default()
            });

            let scale = Mat4::scale_uniform(1.0);

            scale * translation
        };

        for (i, point) in &mut mesh.points.iter_mut().enumerate() {
            point.position = (Vec4::position(point.position) * transform).to_vec3();

            match i {
                0 => {
                    point.position -= vec3::FORWARD * side_length * 0.25;
                }
                1 => {
                    point.position += vec3::FORWARD * side_length * 0.25;
                }
                _ => (),
            }
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

    // Ground collider plane.

    let static_plane_colliders = vec![PlaneCollider::new(Default::default(), vec3::UP)];

    Simulation {
        meshes,
        forces,
        static_plane_colliders,
        ..Default::default()
    }
}
