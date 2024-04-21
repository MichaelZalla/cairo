use crate::{scene::resources::SceneResources, vertex::default_vertex_out::DefaultVertexOut};

use super::context::ShaderContext;

pub type AlphaShaderFn = fn(&ShaderContext, &SceneResources, &DefaultVertexOut) -> bool;
