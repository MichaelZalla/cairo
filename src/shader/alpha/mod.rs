use std::sync::RwLock;

use crate::vertex::default_vertex_out::DefaultVertexOut;

use super::ShaderContext;

pub trait AlphaShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self;
    fn call(&self, out: &DefaultVertexOut) -> bool;
}
