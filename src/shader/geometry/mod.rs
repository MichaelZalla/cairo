use std::sync::RwLock;

use crate::{
    device::{GameControllerState, KeyboardState, MouseState},
    vertex::default_vertex_out::DefaultVertexOut,
};

use self::{options::GeometryShaderOptions, sample::GeometrySample};

use super::ShaderContext;

pub mod options;
pub mod sample;

pub trait GeometryShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>, options: Option<GeometryShaderOptions>) -> Self;

    fn get_options(&self) -> &GeometryShaderOptions;

    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        _mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    );

    fn call(&self, out: &DefaultVertexOut) -> Option<GeometrySample>;
}
