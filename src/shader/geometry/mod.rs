use crate::{scene::resources::SceneResources, vertex::default_vertex_out::DefaultVertexOut};

use self::{options::GeometryShaderOptions, sample::GeometrySample};

pub mod options;
pub mod sample;

use super::context::ShaderContext;

pub type GeometryShaderFn = fn(
    &ShaderContext,
    &SceneResources,
    &GeometryShaderOptions,
    &DefaultVertexOut,
) -> Option<GeometrySample>;
