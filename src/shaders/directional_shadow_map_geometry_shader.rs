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
            world_pos: interpolant.world_pos,
            depth: interpolant.depth,
            ..Default::default()
        })
    };
