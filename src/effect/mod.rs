use crate::{
    device::{GameControllerState, KeyboardState, MouseState},
    material::Material,
};

use super::{color::Color, matrix::Mat4};

pub trait Effect {
    type VertexIn;
    type VertexOut;

    fn get_projection(&self) -> Mat4;

    fn set_projection(&mut self, projection_transform: Mat4);

    fn set_active_material(&mut self, material_option: Option<*const Material>);

    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    );

    fn vs(&self, v: Self::VertexIn) -> Self::VertexOut;

    fn ts(&self, interpolant: &Self::VertexOut) -> bool;

    fn ps(&self, interpolant: &Self::VertexOut) -> Color;
}
