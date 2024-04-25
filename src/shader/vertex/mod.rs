use crate::vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut};

use super::context::ShaderContext;

pub type VertexShaderFn = fn(&ShaderContext, &DefaultVertexIn) -> DefaultVertexOut;
