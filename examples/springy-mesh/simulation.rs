use std::f32::consts::TAU;

use cairo::{
    color,
    geometry::intersect::intersect_line_segment_plane,
    matrix::Mat4,
    physics::simulation::{
        collision_response::resolve_point_plane_collision_approximate,
        force::{gravity::GRAVITY_POINT_FORCE, PointForce},
        state_vector::{FromStateVector, StateVector, ToStateVector},
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

use crate::{
    integration::{integrate_midpoint_euler, system_dynamics_function},
    plane_collider::PlaneCollider,
    springy_mesh::{make_cube, make_springy_mesh, SpringyMesh},
};

pub const COMPONENTS_PER_PARTICLE: usize = 2; // { position, velocity }

#[derive(Default, Debug, Clone)]
pub struct Simulation {
    pub forces: Vec<PointForce>,
    pub meshes: Vec<SpringyMesh>,
    pub static_plane_colliders: Vec<PlaneCollider>,
}

impl Simulation {
    pub fn tick(&mut self, h: f32, uptime_seconds: f32) {
        let num_points: usize = self.meshes.iter().map(|mesh| mesh.points.len()).sum();

        let mut state = StateVector::new(COMPONENTS_PER_PARTICLE, num_points);

        let n = state.len();

        // Copy current positions and velocities into the current state.

        let mut i = 0;

        for mesh in &mut self.meshes {
            mesh.state_index_offset = i;

            for point in &mesh.points {
                point.write_to(&mut state, n, i);

                i += 1;
            }
        }

        // Compute the derivative and integrate over h.

        let derivative =
            system_dynamics_function(&state, &self.forces, &mut self.meshes, uptime_seconds);

        let mut new_state = integrate_midpoint_euler(&state, &derivative, h);

        // Detect and resolve collisions against all static colliders.

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

        // Copy new positions and velocities back into each particle.

        for (i, point) in self
            .meshes
            .iter_mut()
            .flat_map(|mesh| &mut mesh.points)
            .enumerate()
        {
            point.write_from(&new_state, n, i);
        }

        // Update the mesh's AABB bounds.

        for mesh in &mut self.meshes {
            mesh.update_aabb();
        }

        // Update collider triangles for each springy mesh.

        for mesh in &mut self.meshes {
            for triangle in &mut mesh.triangles {
                let (v0, v1, v2) = (
                    &mesh.points[triangle.vertices[0]].position,
                    &mesh.points[triangle.vertices[1]].position,
                    &mesh.points[triangle.vertices[2]].position,
                );

                triangle.update_vertex_positions(v0, v1, v2);
            }
        }
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer) {
        for mesh in &self.meshes {
            // Visualize mesh AABB.

            renderer.render_aabb(&mesh.aabb, Default::default(), color::DARK_GRAY);

            // Visualize points.

            for point in &mesh.points {
                let transform = Mat4::scale(vec3::ONES * 0.1) * Mat4::translation(point.position);

                renderer.render_empty(
                    &transform,
                    EmptyDisplayKind::Sphere(12),
                    false,
                    Some(color::ORANGE),
                );
            }

            // Visualize struts.

            for strut in &mesh.struts {
                // Visualize the strut edge.

                let start = mesh.points[strut.edge.points.0].position;
                let end = mesh.points[strut.edge.points.1].position;

                renderer.render_line(start, end, strut.edge.color);
            }
        }

        // Visualize static plane colliders.
        for collider in &self.static_plane_colliders {
            collider.render(renderer);
        }
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

        meshes.push(mesh);
    }

    // Ground collider plane.

    let static_plane_colliders = vec![PlaneCollider::new(Default::default(), vec3::UP)];

    Simulation {
        meshes,
        forces,
        static_plane_colliders,
    }
}
