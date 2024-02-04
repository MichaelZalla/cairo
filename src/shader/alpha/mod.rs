use std::sync::RwLockReadGuard;

use crate::vertex::default_vertex_out::DefaultVertexOut;

use super::ShaderContext;

pub type AlphaShaderFn = fn(&RwLockReadGuard<'_, ShaderContext>, &DefaultVertexOut) -> bool;
