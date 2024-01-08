use crate::material::Material;

use super::{color::Color, matrix::Mat4};

pub trait Effect {
    type VertexIn;
    type VertexOut;

    fn get_projection(&self) -> Mat4;

    fn set_projection(&mut self, projection_transform: Mat4);

    fn set_active_material(&mut self, material_option: Option<*const Material>);

    fn vs(&self, v: Self::VertexIn) -> Self::VertexOut;

    fn ps(&self, interpolant: &Self::VertexOut) -> Option<Color>;
}
