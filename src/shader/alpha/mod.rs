use crate::vertex::default_vertex_out::DefaultVertexOut;

use super::context::ShaderContext;

pub type AlphaShaderFn = fn(&ShaderContext, &DefaultVertexOut) -> bool;
