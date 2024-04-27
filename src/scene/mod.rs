use crate::app::App;

use super::device::{GameControllerState, KeyboardState, MouseState};

pub mod camera;
pub mod context;
pub mod environment;
pub mod graph;
pub mod light;
pub mod node;
pub mod resources;

pub trait Scene {
    fn update(
        &mut self,
        app: &App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    );

    fn render(&mut self);
}
