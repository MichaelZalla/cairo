use sdl2::sys::SDL_STANDARD_GRAVITY;

use cairo::vec::vec3::Vec3;

use crate::force::{Force, Newtons, Point};
use crate::rigid_body::RigidBody;
use crate::rigid_body_simulation_state::RigidBodySimulationState;
use crate::simulation::Simulation;

pub fn make_simulation() -> Simulation {
    #[allow(unused)]
    let gravity_body_force: Force = Box::new(
        |state: &RigidBodySimulationState, _current_time: f32| -> (Newtons, Option<Point>) {
            static BODY_FORCE: Vec3 = Vec3 {
                x: 0.0,
                y: -(SDL_STANDARD_GRAVITY as f32),
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
