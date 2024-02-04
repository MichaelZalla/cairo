use sdl2::keyboard::Keycode;

use crate::{
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
};

#[derive(Copy, Clone)]
pub struct PipelineOptions {
    pub wireframe_color: Color,
    pub do_wireframe: bool,
    pub do_rasterized_geometry: bool,
    pub do_lighting: bool,
    pub do_visualize_normals: bool,
    pub cull_backfaces: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            wireframe_color: color::WHITE,
            do_wireframe: false,
            do_rasterized_geometry: true,
            do_lighting: true,
            do_visualize_normals: false,
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
                    self.do_wireframe = !self.do_wireframe;
                }
                Keycode::Num2 { .. } => {
                    self.do_rasterized_geometry = !self.do_rasterized_geometry;
                }
                Keycode::Num3 { .. } => {
                    self.do_lighting = !self.do_lighting;
                }
                Keycode::Num4 { .. } => {
                    self.do_visualize_normals = !self.do_visualize_normals;
                }
                Keycode::Num5 { .. } => {
                    self.cull_backfaces = !self.cull_backfaces;
                }
                _ => {}
            }
        }

        if game_controller_state.buttons.x {
            self.do_wireframe = !self.do_wireframe;
        } else if game_controller_state.buttons.y {
            self.do_visualize_normals = !self.do_visualize_normals;
        }
    }
}
