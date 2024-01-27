use sdl2::keyboard::Keycode;

use crate::{
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
};

#[derive(Copy, Clone)]
pub struct PipelineOptions {
    pub wireframe_color: Color,
    pub show_wireframe: bool,
    pub show_rasterized_geometry: bool,
    pub show_lighting: bool,
    pub show_normals: bool,
    pub cull_backfaces: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            wireframe_color: color::WHITE,
            show_wireframe: false,
            show_rasterized_geometry: true,
            show_lighting: true,
            show_normals: false,
            cull_backfaces: true,
        }
    }
}

impl PipelineOptions {
    pub fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Num1 { .. } => {
                    self.show_wireframe = !self.show_wireframe;
                }
                Keycode::Num2 { .. } => {
                    self.show_rasterized_geometry = !self.show_rasterized_geometry;
                }
                Keycode::Num3 { .. } => {
                    self.show_lighting = !self.show_lighting;
                }
                Keycode::Num4 { .. } => {
                    self.show_normals = !self.show_normals;
                }
                Keycode::Num5 { .. } => {
                    self.cull_backfaces = !self.cull_backfaces;
                }
                _ => {}
            }
        }

        if game_controller_state.buttons.x {
            self.show_wireframe = !self.show_wireframe;
        } else if game_controller_state.buttons.y {
            self.show_normals = !self.show_normals;
        }
    }
}
