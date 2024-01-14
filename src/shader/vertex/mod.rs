use std::sync::RwLock;

use crate::vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut};

use super::ShaderContext;

pub trait VertexShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self;

    fn call(&self, v: &DefaultVertexIn) -> DefaultVertexOut;
}
