#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    render::options::shader::RenderShaderOptions,
    scene::{light::POINT_LIGHT_SHADOW_CAMERA_FAR, resources::SceneResources},
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

        let mut out = DefaultVertexOut::new();

        out.position = Vec4::new(v.position, 1.0) * context.world_view_projection_transform;

        let world_pos = Vec4::new(v.position, 1.0) * context.world_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        out
    };

pub static PointShadowMapGeometryShader: GeometryShaderFn = |_context: &ShaderContext,
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

pub static PointShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the linear depth value (in RGB space) for this fragment.

        let distance_to_point_light = (sample.world_pos - context.view_position.to_vec3()).mag();

        let distance_alpha = distance_to_point_light / POINT_LIGHT_SHADOW_CAMERA_FAR;

        Color::from_vec3(Vec3 {
            x: distance_alpha,
            y: distance_alpha,
            z: distance_alpha,
        })
    };

pub static TestPointShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Color {
        // Emit only the linear depth value (in RGB space) for this fragment.

        // @NOTE(mzalla) Hard-codes first point light handle.
        let handle = context.point_lights[0];

        if let Ok(entry) = resources.point_light.borrow().get(&handle) {
            let point_light = &entry.item;

            if let Some(handle) = &point_light.shadow_map {
                if let Ok(entry) = resources.cubemap_f32.borrow().get(handle) {
                    let cubemap = &entry.item;

                    let light_to_fragment = sample.world_pos - point_light.position;

                    let closest_depth =
                        cubemap.sample_nearest(&Vec4::new(light_to_fragment.as_normal(), 1.0));

                    let closest_depth_alpha = closest_depth / POINT_LIGHT_SHADOW_CAMERA_FAR;

                    return Color::from_vec3(Vec3 {
                        x: closest_depth_alpha,
                        y: closest_depth_alpha,
                        z: closest_depth_alpha,
                    });
                }
            }
        }

        Color::from_vec3(vec3::ONES)
    };
