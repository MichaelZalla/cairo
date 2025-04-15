use cairo::{
    physics::simulation::{particle::Particle, state_vector::StateVector, units::Newtons},
    vec::vec3::{self, Vec3},
};

pub static STRENGTH_PER_UNIT_LENGTH: f32 = 400.0;

pub static DAMPER_PER_UNIT_LENGTH: f32 = 250.0;

#[derive(Default, Debug, Copy, Clone)]
pub struct Strut {
    pub strength: f32,
    pub damper: f32,
    pub rest_length: f32,
    pub points: (usize, usize),
    pub delta_length: f32,
    pub is_internal: bool,
}

impl Strut {
    pub fn new(i: usize, j: usize, points: &[Particle], is_internal: bool) -> Self {
        let rest_length = (points[j].position - points[i].position).mag();

        Strut {
            points: (i, j),
            rest_length,
            strength: STRENGTH_PER_UNIT_LENGTH / rest_length,
            damper: DAMPER_PER_UNIT_LENGTH / rest_length,
            is_internal,
            ..Default::default()
        }
    }

    pub fn compute_accelerations(
        &mut self,
        current_state: &StateVector,
        derivative: &mut StateVector,
        state_index_offset: usize,
        n: usize,
        wind: &Vec3,
    ) {
        let i = self.points.0;
        let j = self.points.1;

        // For each strut, add spring and damper forces to connected points.

        let spring_force_i_j = self.compute_spring_force(current_state, state_index_offset, n);

        // If using air resistance, compute lift and drag for each strut;
        // distribute forces to connected points.

        // Local wind velocity vector.

        let (drag_force, lift_force) =
            self.compute_drag_and_lift_forces(current_state, state_index_offset, n, wind);

        // Combine forces to determine a net force.

        static POINT_MASS: f32 = 2.5;

        let strut_mass = POINT_MASS * 2.0;

        let spring_acceleration_i_j = spring_force_i_j / POINT_MASS;

        let drag_lift_acceleration_per_point = ((drag_force + lift_force) / strut_mass) * 0.5;

        derivative.data[state_index_offset + i + n] += spring_acceleration_i_j;
        derivative.data[state_index_offset + j + n] -= spring_acceleration_i_j;

        derivative.data[state_index_offset + i + n] += drag_lift_acceleration_per_point;
        derivative.data[state_index_offset + j + n] += drag_lift_acceleration_per_point;
    }

    fn compute_spring_force(
        &mut self,
        current_state: &StateVector,
        state_index_offset: usize,
        n: usize,
    ) -> Newtons {
        let i = self.points.0;
        let j = self.points.1;

        let point_i = current_state.data[state_index_offset + i];
        let point_j = current_state.data[state_index_offset + j];

        let i_j = point_j - point_i;
        let i_j_direction = i_j.as_normal();
        let i_j_distance = i_j.mag();

        self.delta_length = i_j_distance - self.rest_length;

        let spring_force_i_j = i_j_direction * self.strength * self.delta_length;

        let difference_in_velocities_along_strut = (current_state.data[state_index_offset + j + n]
            - current_state.data[state_index_offset + i + n])
            .dot(i_j_direction);

        let damper_force_i_j = i_j_direction * self.damper * difference_in_velocities_along_strut;

        spring_force_i_j + damper_force_i_j
    }

    fn compute_drag_and_lift_forces(
        &self,
        current_state: &StateVector,
        state_index_offset: usize,
        n: usize,
        wind: &Vec3,
    ) -> (Newtons, Newtons) {
        static DRAG_COEFFICIENT: f32 = 0.2;
        static LIFT_COEFFICIENT: f32 = 0.4;

        let i = self.points.0;
        let j = self.points.1;

        let point_i = current_state.data[state_index_offset + i];
        let point_j = current_state.data[state_index_offset + j];

        let i_j = point_j - point_i;
        let i_j_direction = i_j.as_normal();
        let i_j_distance = i_j.mag();

        let tangent = i_j_direction;
        let normal = vec3::FORWARD.cross(tangent).as_normal();

        let velocity_i = current_state.data[state_index_offset + i + n];
        let velocity_j = current_state.data[state_index_offset + j + n];

        // Average of the two points' velocities.

        let midpoint_velocity = (velocity_i + velocity_j) / 2.0;

        // Average velocity, relative to current wind velocity.

        let relative_midpoint_velocity = midpoint_velocity - *wind;

        // Effective "length" of the strut that the air can push against, scaled
        // by the magnitude of the (relative) midpoint velocity.

        let drag_force = {
            let effective_length = {
                if relative_midpoint_velocity.is_zero() {
                    Default::default()
                } else {
                    let relative_midpoint_velocity_direction =
                        relative_midpoint_velocity.as_normal();

                    let n_dot_v_r = normal.dot(relative_midpoint_velocity_direction).abs();

                    i_j_distance * n_dot_v_r
                }
            };

            -relative_midpoint_velocity * DRAG_COEFFICIENT * effective_length
        };

        let lift_force = {
            let n_cross_v_r = normal.cross(relative_midpoint_velocity);

            if n_cross_v_r.is_zero() {
                Default::default()
            } else {
                let n_cross_v_r_unit = n_cross_v_r.as_normal();

                relative_midpoint_velocity.cross(n_cross_v_r_unit) * -LIFT_COEFFICIENT
            }
        };

        (drag_force, lift_force)
    }
}
