use std::sync::RwLock;

use sdl2::keyboard::Keycode;

use crate::{
    color::Color,
    device::{GameControllerState, KeyboardState, MouseState},
    vertex::default_vertex_out::DefaultVertexOut,
};

use super::ShaderContext;

#[derive(Debug)]
pub struct FragmentShaderOptions {
    pub bilinear_active: bool,
    pub ambient_occlusion_mapping_active: bool,
    pub diffuse_mapping_active: bool,
    pub normal_mapping_active: bool,
    pub specular_mapping_active: bool,
    pub emissive_mapping_active: bool,
}

impl Default for FragmentShaderOptions {
    fn default() -> Self {
        Self {
            bilinear_active: false,
            ambient_occlusion_mapping_active: false,
            diffuse_mapping_active: true,
            normal_mapping_active: false,
            specular_mapping_active: false,
            emissive_mapping_active: false,
        }
    }
}

impl FragmentShaderOptions {
    pub fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        _game_controller_state: &GameControllerState,
    ) {
        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::B { .. } => {
                    self.bilinear_active = !self.bilinear_active;
                }
                Keycode::O { .. } => {
                    self.ambient_occlusion_mapping_active = !self.ambient_occlusion_mapping_active;
                }
                Keycode::P { .. } => {
                    self.diffuse_mapping_active = !self.diffuse_mapping_active;
                }
                Keycode::N { .. } => {
                    self.normal_mapping_active = !self.normal_mapping_active;
                }
                Keycode::M { .. } => {
                    self.specular_mapping_active = !self.specular_mapping_active;
                }
                Keycode::K { .. } => {
                    self.emissive_mapping_active = !self.emissive_mapping_active;
                }
                _ => {}
            }
        }
    }
}

pub trait FragmentShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<FragmentShaderOptions>) -> Self;

    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    );

    fn call(&self, out: &DefaultVertexOut) -> Color;
}
