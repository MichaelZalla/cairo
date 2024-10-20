#![allow(non_upper_case_globals)]

use crate::{
    render::options::shader::RenderShaderOptions,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext,
        geometry::{sample::GeometrySample, GeometryShaderFn},
    },
    vertex::default_vertex_out::DefaultVertexOut,
};

pub static DirectionalShadowMapGeometryShader: GeometryShaderFn =
    |_context: &ShaderContext,
     _resources: &SceneResources,
     _options: &RenderShaderOptions,
     interpolant: &DefaultVertexOut|
     -> Option<GeometrySample> {
        Some(GeometrySample {
            stencil: true,
            position_world_space: interpolant.position_world_space,
            depth: interpolant.depth,
            alpha: 1.0,
            ..Default::default()
        })
    };
