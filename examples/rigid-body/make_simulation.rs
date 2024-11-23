use cairo::{
    physics::simulation::{force::ContactPoint, physical_constants::EARTH_GRAVITY, units::Newtons},
    vec::vec3::Vec3,
};

use crate::{
    rigid_body::RigidBody,
    rigid_body_simulation_state::RigidBodySimulationState,
    simulation::{RigidBodyForce, Simulation},
};

pub fn make_simulation() -> Simulation {
    #[allow(unused)]
    let gravity_body_force: RigidBodyForce = Box::new(
        |state: &RigidBodySimulationState,
         _i: usize,
         _current_time: f32|
         -> (Newtons, Option<ContactPoint>) {
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
