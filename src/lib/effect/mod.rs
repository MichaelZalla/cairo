use crate::vertices::default_vertex::DefaultVertex;

use super::{color::Color, vec::vec3::Vec3};

pub trait Effect<T = DefaultVertex> {

	type Vertex;

	fn get_rotation(&self) -> Vec3;

	fn vs(&self, v: T) -> Self::Vertex;

	fn ps(&self, interpolant: Self::Vertex) -> Color;

}
