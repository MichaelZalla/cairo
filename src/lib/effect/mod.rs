use crate::vertices::default_vertex::DefaultVertex;

pub trait Effect<T = DefaultVertex> {

	type Vertex;

	fn ps(&self, interpolant: Self::Vertex) -> ();

}
