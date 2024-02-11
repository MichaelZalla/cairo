use std::sync::RwLock;

use sdl2::keyboard::Keycode;

use crate::{
    device::{GameControllerState, KeyboardState, MouseState},
    vertex::default_vertex_out::DefaultVertexOut,
};

use self::sample::GeometrySample;

use super::ShaderContext;

pub mod sample;

#[derive(Debug)]
pub struct GeometryShaderOptions {
    pub bilinear_active: bool,
    pub trilinear_active: bool,
    pub ambient_occlusion_mapping_active: bool,
    pub diffuse_mapping_active: bool,
    pub normal_mapping_active: bool,
    pub specular_mapping_active: bool,
    pub emissive_mapping_active: bool,
}

impl Default for GeometryShaderOptions {
    fn default() -> Self {
        Self {
            bilinear_active: false,
            trilinear_active: false,
            ambient_occlusion_mapping_active: false,
            diffuse_mapping_active: true,
            normal_mapping_active: false,
            specular_mapping_active: false,
            emissive_mapping_active: false,
        }
    }
}

impl GeometryShaderOptions {
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

                    println!("Bilinear filtering: {}", self.bilinear_active)
                }
                Keycode::T { .. } => {
                    self.trilinear_active = !self.trilinear_active;

                    println!("Trilinear filtering: {}", self.trilinear_active)
                }
                Keycode::O { .. } => {
                    self.ambient_occlusion_mapping_active = !self.ambient_occlusion_mapping_active;

                    println!(
                        "ambient_occlusion_mapping_active: {}",
                        self.ambient_occlusion_mapping_active
                    )
                }
                Keycode::P { .. } => {
                    self.diffuse_mapping_active = !self.diffuse_mapping_active;

                    println!(
                        "Diffuse mapping: {}",
                        if self.diffuse_mapping_active {
                            "On"
                        } else {
                            "Off"
                        }
                    )
                }
                Keycode::N { .. } => {
                    self.normal_mapping_active = !self.normal_mapping_active;

                    println!(
                        "Normal mapping: {}",
                        if self.normal_mapping_active {
                            "On"
                        } else {
                            "Off"
                        }
                    )
                }
                Keycode::M { .. } => {
                    self.specular_mapping_active = !self.specular_mapping_active;

                    println!(
                        "Specular mapping: {}",
                        if self.specular_mapping_active {
                            "On"
                        } else {
                            "Off"
                        }
                    )
                }
                Keycode::K { .. } => {
                    self.emissive_mapping_active = !self.emissive_mapping_active;

                    println!(
                        "Emissive mapping: {}",
                        if self.emissive_mapping_active {
                            "On"
                        } else {
                            "Off"
                        }
                    )
                }
                _ => {}
            }
        }
    }
}

pub trait GeometryShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<GeometryShaderOptions>) -> Self;

    fn get_options(&self) -> &GeometryShaderOptions;

    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    );

    fn call(&self, out: &DefaultVertexOut) -> GeometrySample;
}
