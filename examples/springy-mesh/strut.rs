use cairo::{
    color::Color,
    physics::simulation::{particle::Particle, state_vector::StateVector, units::Newtons},
    vec::vec3::Vec3,
};

pub static PARTICLE_MASS: f32 = 2.0;

static STRENGTH_PER_UNIT_LENGTH: f32 = 1_000.0;
static DAMPER_PER_UNIT_LENGTH: f32 = 150.0;
static TORSIONAL_STRENGTH: f32 = 100.0;
static TORSIONAL_DAMPER: f32 = 10.0;

#[derive(Default, Debug, Clone)]
pub struct Edge {
    pub points: (usize, usize),
    #[allow(unused)]
    pub connected_points: Option<(usize, usize)>,
    pub color: Color,
}

#[derive(Default, Debug, Clone)]
pub struct Strut {
    pub edge: Edge,
    pub spring_strength: f32,
    pub spring_damper: f32,
    pub rest_length: f32,
    pub delta_length: f32,
    pub torsional_strength: f32,
    pub torsional_damper: f32,
    pub rest_angle: f32,
    pub delta_angle: f32,
}

impl Strut {
    pub fn new(points: &[Particle], edge: Edge) -> Self {
        let a = points[edge.points.0].position;
        let b = points[edge.points.1].position;

        let a_b = b - a;

        // Computes rest length of strut.

        let rest_length = a_b.mag();

        let rest_angle =
            if let Some((left_normal, right_normal)) = Self::get_surface_normals(&edge, points) {
                // Computes rest angle of torsional spring.

                let h = {
                    let start = points[edge.points.0].position;
                    let end = points[edge.points.1].position;

                    (end - start).as_normal()
                };

                Self::get_angle(h, left_normal, right_normal)
            } else {
                0.0
            };

        Self {
            edge,
            spring_strength: STRENGTH_PER_UNIT_LENGTH / rest_length,
            spring_damper: DAMPER_PER_UNIT_LENGTH / rest_length,
            rest_length,
            torsional_strength: TORSIONAL_STRENGTH,
            torsional_damper: TORSIONAL_DAMPER,
            rest_angle,
            ..Default::default()
        }
    }

    pub fn get_surface_normals(edge: &Edge, points: &[Particle]) -> Option<(Vec3, Vec3)> {
        if let Some(connected_points) = &edge.connected_points {
            let start = points[edge.points.0].position;
            let end = points[edge.points.1].position;
            let left = points[connected_points.0].position;
            let right = points[connected_points.1].position;

            Some((
                (left - start).cross(end - left).as_normal(),
                (right - end).cross(start - right).as_normal(),
            ))
        } else {
            None
        }
    }

    pub fn get_surface_normals_state_vector(
        edge: &Edge,
        state: &StateVector,
        state_index_offset: usize,
    ) -> Option<(Vec3, Vec3)> {
        if let Some(connected_points) = &edge.connected_points {
            let start = state.data[state_index_offset + edge.points.0];
            let end = state.data[state_index_offset + edge.points.1];
            let left = state.data[state_index_offset + connected_points.0];
            let right = state.data[state_index_offset + connected_points.1];

            Some((
                (left - start).cross(end - left).as_normal(),
                (right - end).cross(start - right).as_normal(),
            ))
        } else {
            None
        }
    }

    pub fn get_angle(h: Vec3, left_normal: Vec3, right_normal: Vec3) -> f32 {
        let cosine_theta = -right_normal.dot(-left_normal);

        let sine_theta = -right_normal.cross(-left_normal).dot(h);

        sine_theta.atan2(cosine_theta)
    }

    pub fn compute_accelerations(
        &mut self,
        current_state: &StateVector,
        derivative: &mut StateVector,
        state_index_offset: usize,
        n: usize,
    ) {
        let start_index = self.edge.points.0;
        let end_index = self.edge.points.1;

        // Linear spring accelerations.

        let spring_force = self.compute_spring_force(current_state, state_index_offset, n);

        let spring_acceleration = spring_force / PARTICLE_MASS;

        // { mesh_start + mesh_point_index + acceleration_component_index }

        derivative.data[state_index_offset + start_index + n] += spring_acceleration;
        derivative.data[state_index_offset + end_index + n] -= spring_acceleration;

        if let Some(connected_points) = &self.edge.connected_points {
            let left_index = connected_points.0;
            let right_index = connected_points.1;

            let (f0, f1, f2, f3) = self.compute_rotational_forces(
                current_state,
                state_index_offset,
                n,
                (start_index, end_index, left_index, right_index),
            );

            derivative.data[state_index_offset + start_index + n] += f0;
            derivative.data[state_index_offset + end_index + n] += f1;
            derivative.data[state_index_offset + left_index + n] += f2;
            derivative.data[state_index_offset + right_index + n] += f3;
        }
    }

    fn compute_spring_force(
        &mut self,
        current_state: &StateVector,
        state_index_offset: usize,
        n: usize,
    ) -> Newtons {
        let i = self.edge.points.0;
        let j = self.edge.points.1;

        // Reads current point positions.

        let position_i = current_state.data[state_index_offset + i];
        let velocity_i = current_state.data[state_index_offset + i + n];

        let position_j = current_state.data[state_index_offset + j];
        let velocity_j = current_state.data[state_index_offset + j + n];

        // Computes the strut vector and its distance.

        let i_j = position_j - position_i;

        let i_j_direction = i_j.as_normal();

        let i_j_distance = i_j.mag();

        // Compares the strut's current length with its rest length.
        // Caches the delta so we can visualize it.

        self.delta_length = i_j_distance - self.rest_length;

        // Spring force applied in the direction of the strut, scaled linearly
        // by the current length delta, multiplied by a strength coefficient.

        let spring_force = i_j_direction * self.spring_strength * self.delta_length;

        // Computes the two points' velocity delta, along the strut direction.

        let difference_in_velocities_along_strut = (velocity_j - velocity_i).dot(i_j_direction);

        // Damper force applied in the direction of the strut, scaled by how
        // quickly the two point are moving towards each other or away from each
        // other, multiplied by a damper strength coefficient.

        let damper_force =
            i_j_direction * self.spring_damper * difference_in_velocities_along_strut;

        // Computes net force from spring and damper forces (acting in opposite
        // directions).

        spring_force + damper_force
    }

    fn compute_rotational_forces(
        &mut self,
        current_state: &StateVector,
        state_index_offset: usize,
        n: usize,
        vertex_indices: (usize, usize, usize, usize),
    ) -> (Vec3, Vec3, Vec3, Vec3) {
        let start_index = vertex_indices.0;
        let end_index = vertex_indices.1;
        let left_index = vertex_indices.2;
        let right_index = vertex_indices.3;

        let start = current_state.data[state_index_offset + start_index];
        let end = current_state.data[state_index_offset + end_index];

        let left = current_state.data[state_index_offset + left_index];
        let right = current_state.data[state_index_offset + right_index];

        // Angular spring accelerations.

        let left_velocity = current_state.data[state_index_offset + left_index + n];
        let right_velocity = current_state.data[state_index_offset + right_index + n];

        let start_end = end - start;

        let h = start_end.as_normal();

        let start_left = left - start;
        let start_right = right - start;

        let left_r = start_left - h * start_left.dot(h);
        let left_r_mag = left_r.mag();

        let right_r = start_right - h * start_right.dot(h);
        let right_r_mag = right_r.mag();

        let (left_normal, right_normal) =
            Strut::get_surface_normals_state_vector(&self.edge, current_state, state_index_offset)
                .unwrap();

        // Computes the current spring angle.

        let angle = Strut::get_angle(h, left_normal, right_normal);

        self.delta_angle = angle - self.rest_angle;

        let torque_magnitude = self.torsional_strength * self.delta_angle;

        // Torsional spring damper.

        // The connected point's speed along the surface normal.
        let left_s = left_velocity.dot(-left_normal);
        let right_s = right_velocity.dot(-right_normal);

        let left_angular_speed_radians = left_s / left_r_mag;
        let right_angular_speed_radians = right_s / right_r_mag;

        let damper_magnitude =
            -self.torsional_damper * (left_angular_speed_radians + right_angular_speed_radians);

        // Net torque.

        let torque = h * (torque_magnitude + damper_magnitude.max(-torque_magnitude));

        // Compute the forces acting on the 4 independent particles.

        let torque_dot_h = torque.dot(h);

        let f2 = left_normal * (torque_dot_h / right_r_mag);
        let f3 = right_normal * (torque_dot_h / left_r_mag);

        let left_d = start_left.dot(h);
        let right_d = start_right.dot(h);

        let l = start_end.mag();

        let f1 = -(f2 * left_d + f3 * right_d) / l;

        let f0 = -(f1 + f2 + f3);

        (f0, f1, f2, f3)
    }
}
