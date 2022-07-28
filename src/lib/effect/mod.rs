use crate::vertices::default_vertex::DefaultVertex;

use super::color::Color;

pub trait Effect<T = DefaultVertex> {

	type Vertex;

	fn vs(&self, v: T) -> Self::Vertex;

	fn ps(&self, interpolant: Self::Vertex) -> Color;

}
