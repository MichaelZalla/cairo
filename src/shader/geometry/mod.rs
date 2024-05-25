use crate::{
    render::options::shader::RenderShaderOptions, scene::resources::SceneResources,
    vertex::default_vertex_out::DefaultVertexOut,
};

use self::sample::GeometrySample;

pub mod sample;

use super::context::ShaderContext;

pub type GeometryShaderFn = fn(
    &ShaderContext,
    &SceneResources,
    &RenderShaderOptions,
    &DefaultVertexOut,
) -> Option<GeometrySample>;
