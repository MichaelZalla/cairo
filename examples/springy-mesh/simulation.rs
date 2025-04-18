use cairo::{
    geometry::primitives::line_segment::LineSegment,
    physics::simulation::{
        collision_response::resolve_plane_collision_approximate,
        collision_test::test_line_segment_plane,
        force::{ContactPoint, PointForce},
        physical_constants::EARTH_GRAVITY,
        state_vector::{FromStateVector, StateVector, ToStateVector},
        units::Newtons,
    },
    random::sampler::RandomSampler,
    vec::vec3,
};

use crate::{
    integration::{integrate_euler, system_dynamics_function},
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

        // Compute the derivative and integrate over the time delta.

        let derivative =
            system_dynamics_function(&state, &self.forces, &mut self.meshes, uptime_seconds);

        let mut new_state = integrate_euler(&state, &derivative, h);

        // Detect and resolve collisions against all static colliders.

        for mesh in &self.meshes {
            for i in 0..mesh.points.len() {
                let start_position = state.data[mesh.state_index_offset + i];
                let mut end_position = new_state.data[mesh.state_index_offset + i];
                let mut end_velocity = new_state.data[mesh.state_index_offset + i + n];

                // We'll break early on the first collision (if any).

                let segment = LineSegment::new(start_position, end_position);

                for collider in &self.static_plane_colliders {
                    if let Some((_f, penetration_depth)) =
                        test_line_segment_plane(&segment, &collider.plane)
                    {
                        resolve_plane_collision_approximate(
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
}

static GRAVITY_POINT_FORCE: PointForce =
    |_state: &StateVector, _i: usize, _current_time: f32| -> (Newtons, Option<ContactPoint>) {
        let newtons = -vec3::UP * EARTH_GRAVITY;

        (newtons, None)
    };

pub fn make_simulation(sampler: &mut RandomSampler<1024>) -> Simulation {
    // Forces.

    let forces: Vec<PointForce> = vec![GRAVITY_POINT_FORCE];

    // Springy meshes.

    let mut meshes = Vec::with_capacity(100);

    for _ in 0..100 {
        let (points, struts) = make_cube(3.0);

        meshes.push(make_springy_mesh(points, struts, sampler));
    }

    // Ground collider plane.

    let static_plane_colliders = vec![PlaneCollider::new(Default::default(), vec3::UP)];

    Simulation {
        meshes,
        forces,
        static_plane_colliders,
    }
}
