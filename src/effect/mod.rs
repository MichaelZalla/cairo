use super::{
	matrix::Mat4,
	color::Color,
};

pub trait Effect {

	type VertexIn;
	type VertexOut;

	fn get_projection(&self) -> Mat4;

	fn vs(&self, v: Self::VertexIn) -> Self::VertexOut;

	fn ps(&self, interpolant: &Self::VertexOut) -> Color;

}