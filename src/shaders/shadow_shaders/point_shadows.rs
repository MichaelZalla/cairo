#![allow(non_upper_case_globals)]

use crate::{
    render::options::shader::RenderShaderOptions,
    scene::{light::shadow::DEFAULT_SHADOW_MAP_CAMERA_FAR, resources::SceneResources},
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

pub static PointShadowMapVertexShader: VertexShaderFn =
    |context: &ShaderContext, v: &DefaultVertexIn| -> DefaultVertexOut {
        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut {
            position_projection_space: Vec4::new(v.position, 1.0)
                * context.world_view_projection_transform,
            ..Default::default()
        };

        let world_pos = Vec4::new(v.position, 1.0) * context.world_transform;

        out.position_world_space = world_pos.to_vec3();

        out
    };

pub static PointShadowMapGeometryShader: GeometryShaderFn = |_context: &ShaderContext,
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

pub static PointShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        // Emit only the linear depth value (in RGB space) for this fragment.

        let distance_to_point_light =
            (sample.position_world_space - context.view_position.to_vec3()).mag();

        let projection_z_far = context
            .projection_z_far
            .unwrap_or(DEFAULT_SHADOW_MAP_CAMERA_FAR);

        let distance_alpha = distance_to_point_light / projection_z_far;

        vec3::ONES * distance_alpha
    };
