use crate::{
	lib::{
		effect::Effect,
	},
	vertices::default_vertex::DefaultVertex
};

pub struct DefaultEffect {}

impl Effect for DefaultEffect {

	type Vertex = DefaultVertex;

	fn ps(&self, interpolant: <Self as Effect>::Vertex) -> () {
		// @TODO(mzalla) Implement pixel shader
	}

}
