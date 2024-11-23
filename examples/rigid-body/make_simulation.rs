use cairo::{physics::simulation::physical_constants::EARTH_GRAVITY, vec::vec3::Vec3};

use crate::{
    force::{Force, Newtons, Point},
    rigid_body::RigidBody,
    rigid_body_simulation_state::RigidBodySimulationState,
    simulation::Simulation,
};

pub fn make_simulation() -> Simulation {
    #[allow(unused)]
    let gravity_body_force: Force = Box::new(
        |state: &RigidBodySimulationState, _current_time: f32| -> (Newtons, Option<Point>) {
            static BODY_FORCE: Vec3 = Vec3 {
                x: 0.0,
                y: -EARTH_GRAVITY,
                z: 0.0,
            };

            (BODY_FORCE / state.inverse_mass, None)
        },
    );

    let forces = vec![
        //
        // gravity_body_force,
    ];

    let rigid_bodies = vec![RigidBody::circle(Default::default(), 5.0, 2.5)];

    Simulation {
        forces,
        rigid_bodies,
    }
}
