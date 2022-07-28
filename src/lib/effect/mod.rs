use super::color::Color;

pub trait Effect {

	type VertexIn;
	type VertexOut;

	fn vs(&self, v: Self::VertexIn) -> Self::VertexOut;

	fn ps(&self, interpolant: Self::VertexOut) -> Color;

}
