#![allow(non_upper_case_globals)]

use crate::{
    render::options::shader::RenderShaderOptions,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext,
        fragment::FragmentShaderFn,
        geometry::{sample::GeometrySample, GeometryShaderFn},
        vertex::VertexShaderFn,
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub static DirectionalShadowMapVertexShader: VertexShaderFn =
    |context: &ShaderContext, v: &DefaultVertexIn| -> DefaultVertexOut {
        let position_vec4 = Vec4::new(v.position, 1.0);

        DefaultVertexOut {
            position_projection_space: position_vec4 * context.world_view_projection_transform,
            position_world_space: (position_vec4 * context.world_transform).to_vec3(),
            ..Default::default()
        }
    };

pub static DirectionalShadowMapGeometryShader: GeometryShaderFn =
    |_context: &ShaderContext,
     _resources: &SceneResources,
     _options: &RenderShaderOptions,
     interpolant: &DefaultVertexOut|
     -> Option<GeometrySample> {
        Some(GeometrySample {
            position_world_space: interpolant.position_world_space,
            depth: interpolant.depth,
            alpha: 1.0,
            ..Default::default()
        })
    };

pub static DirectionalShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        let fragment_position_projection_space = Vec4::new(sample.position_world_space, 1.0)
            * context.view_inverse_transform
            * context.projection_transform;

        let fragment_depth_ndc_space =
            fragment_position_projection_space.z / fragment_position_projection_space.w;

        vec3::ONES * fragment_depth_ndc_space
    };
