use cairo::{
    color::Color,
    physics::simulation::{particle::Particle, state_vector::StateVector, units::Newtons},
};

pub static PARTICLE_MASS: f32 = 10.0;

static STRENGTH_PER_UNIT_LENGTH: f32 = 1750.0;

static DAMPER_PER_UNIT_LENGTH: f32 = 300.0;

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
}

impl Strut {
    pub fn new(points: &[Particle], edge: Edge) -> Self {
        let a = points[edge.points.0].position;
        let b = points[edge.points.1].position;

        let a_b = b - a;

        // Computes rest length of strut.

        let rest_length = a_b.mag();

        Self {
            edge,
            spring_strength: STRENGTH_PER_UNIT_LENGTH / rest_length,
            spring_damper: DAMPER_PER_UNIT_LENGTH / rest_length,
            rest_length,
            ..Default::default()
        }
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
}
