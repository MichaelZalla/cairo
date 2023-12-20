use super::device::{GameControllerState, KeyboardState, MouseState};

pub mod camera;
pub mod light;

pub trait Scene {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    );

    fn render(&mut self);

    fn get_pixel_data(&self) -> &Vec<u32>;
}
