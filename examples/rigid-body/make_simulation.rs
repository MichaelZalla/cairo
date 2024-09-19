use sdl2::sys::SDL_STANDARD_GRAVITY;

use cairo::vec::vec3::Vec3;

use crate::force::{Force, Newtons};
use crate::rigid_body::RigidBody;
use crate::simulation::Simulation;
use crate::state_vector::StateVector;

static GRAVITY: Force = |_state: &StateVector, _i: usize, _current_time: f32| -> Newtons {
    Vec3 {
        x: 0.0,
        y: -(SDL_STANDARD_GRAVITY as f32),
        z: 0.0,
    }
};

pub fn make_simulation<'a>() -> Simulation<'a> {
    let rigid_bodies = vec![RigidBody::circle(Default::default(), 5.0, 2.5)];

    Simulation {
        forces: vec![&GRAVITY],
        rigid_bodies,
    }
}
