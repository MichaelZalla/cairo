use sdl2::keyboard::Keycode;

use crate::{
    color::{self, Color},
    device::{GameControllerState, KeyboardState, MouseState},
};

#[derive(Copy, Clone)]
pub struct PipelineOptions {
    pub wireframe_color: Color,
    pub should_render_wireframe: bool,
    pub should_render_shader: bool,
    pub should_render_normals: bool,
    pub should_cull_backfaces: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            wireframe_color: color::WHITE,
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
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
                    self.should_render_wireframe = !self.should_render_wireframe;
                }
                Keycode::Num2 { .. } => {
                    self.should_render_shader = !self.should_render_shader;
                }
                Keycode::Num3 { .. } => {
                    self.should_render_normals = !self.should_render_normals;
                }
                Keycode::Num4 { .. } => {
                    self.should_cull_backfaces = !self.should_cull_backfaces;
                }
                _ => {}
            }
        }

        if game_controller_state.buttons.x {
            self.should_render_wireframe = !self.should_render_wireframe;
        } else if game_controller_state.buttons.y {
            self.should_render_normals = !self.should_render_normals;
        }
    }
}
