use crate::{
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    time::TimingInfo,
};

use super::LookVector;

pub mod first_person;

pub trait LookVectorController {
    fn update(
        &mut self,
        look_vector: &mut LookVector,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: Option<&MouseState>,
        game_controller_state: &GameControllerState,
        movement_speed: f32,
    );
}
