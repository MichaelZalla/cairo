use cairo::{
    physics::simulation::{
        force::ContactPoint,
        physical_constants::EARTH_GRAVITY,
        rigid_body::{
            rigid_body_simulation_state::{DynRigidBodyForce, RigidBodySimulationState},
            RigidBody, RigidBodyKind,
        },
        units::Newtons,
    },
    vec::vec3::Vec3,
};

use crate::simulation::Simulation;

pub fn make_simulation() -> Simulation {
    #[allow(unused)]
    let gravity_body_force: Box<DynRigidBodyForce> = Box::new(
        |state: &RigidBodySimulationState,
         _i: usize,
         _current_time: f32|
         -> (Newtons, Option<ContactPoint>) {
            static ACCELERATION: Vec3 = Vec3 {
                x: 0.0,
                y: -EARTH_GRAVITY,
                z: 0.0,
            };

            (ACCELERATION / state.inverse_mass, None)
        },
    );

    let forces = vec![
        // gravity_body_force,
    ];

    let rigid_bodies = vec![RigidBody::new(
        RigidBodyKind::Circle(2.5),
        1.0,
        Default::default(),
    )];

    Simulation {
        forces,
        rigid_bodies,
    }
}
