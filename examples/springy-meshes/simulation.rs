use cairo::vec::vec3::Vec3;

use crate::{
    force::Force,
    springy_mesh::SpringyMesh,
    state_vector::{FromStateVector, StateVector, ToStateVector},
};

pub struct Simulation<'a> {
    pub forces: Vec<&'a Force>,
    pub wind: Vec3,
    pub mesh: SpringyMesh,
}

impl<'a> Simulation<'a> {
    pub fn tick(&mut self, current_time: f32, h: f32) {
        let n = self.mesh.points.len();

        let mut state = StateVector::new(2, n);

        for (i, point) in self.mesh.points.iter().enumerate() {
            point.write_to(&mut state, n, i);
        }

        let derivative = self.system_dynamics_function(&state, current_time, h);

        let new_state = self.integrate(&state, &derivative, h);

        for (i, point) in self.mesh.points.iter_mut().enumerate() {
            point.write_from(&new_state, n, i);
        }
    }

    fn system_dynamics_function(
        &self,
        current_state: &StateVector,
        current_time: f32,
        h: f32,
    ) -> StateVector {
        let n = current_state.len();

        // Compute new accelerations (i.e., derivative).
        let mut derivative = self.compute_accelerations(current_state, current_time, h);

        for i in 0..n {
            // Copy velocities from previous (current?) state.
            derivative.data[i] = current_state.data[i + n];
        }

        derivative
    }

    fn compute_accelerations(
        &self,
        current_state: &StateVector,
        current_time: f32,
        _h: f32,
    ) -> StateVector {
        let n = current_state.len();

        let mut derivative = StateVector::new(2, n);

        // For each point, compute net environmental force acting on it.
        for i in 0..n {
            let mut net_force_acceleration: Vec3 = Default::default();

            for force in &self.forces {
                if i == 0 {
                    continue;
                }

                net_force_acceleration += force(current_state, i, current_time);
            }

            // Write the final net environmental acceleration.
            derivative.data[i + n] = net_force_acceleration;
        }

        // Compute forces acting on the mesh (spring, damper, drag, and lift).
        for strut in &self.mesh.struts {
            strut.compute_accelerations(&current_state, &mut derivative, n, &self.wind);
        }

        derivative
    }

    fn integrate(
        &self,
        current_state: &StateVector,
        derivative: &StateVector,
        h: f32,
    ) -> StateVector {
        // Performs basic Euler integration over position and velocity.

        current_state.clone() + derivative * h
    }
}
