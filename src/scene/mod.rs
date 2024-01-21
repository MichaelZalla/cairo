use crate::time::TimingInfo;

use super::device::{GameControllerState, KeyboardState, MouseState};

pub mod camera;
pub mod light;

pub trait Scene {
    fn update(
        &mut self,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    );

    fn render(&mut self);

    fn get_pixel_data(&self) -> &Vec<u32>;
}
