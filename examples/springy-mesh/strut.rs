use cairo::physics::simulation::{state_vector::StateVector, units::Newtons};

#[derive(Default, Debug, Clone)]
pub struct Strut {
    pub spring_strength: f32,
    pub spring_damper: f32,
    pub rest_length: f32,
    pub points: (usize, usize),
    pub delta_length: f32,
}

impl Strut {
    pub fn compute_accelerations(
        &mut self,
        current_state: &StateVector,
        derivative: &mut StateVector,
        state_index_offset: usize,
        n: usize,
    ) {
        let spring_force = self.compute_spring_force(current_state, state_index_offset, n);

        let spring_acceleration_i_j = spring_force / 10.0;

        let i = self.points.0;
        let j = self.points.1;

        // { mesh_start + mesh_point_index + acceleration_component_index }

        derivative.data[state_index_offset + i + n] += spring_acceleration_i_j;
        derivative.data[state_index_offset + j + n] -= spring_acceleration_i_j;
    }

    fn compute_spring_force(
        &mut self,
        current_state: &StateVector,
        state_index_offset: usize,
        n: usize,
    ) -> Newtons {
        let i = self.points.0;
        let j = self.points.1;

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
