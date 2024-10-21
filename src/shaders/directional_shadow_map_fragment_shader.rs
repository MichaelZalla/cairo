#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::{
        light::shadow::{DEFAULT_SHADOW_MAP_CAMERA_FAR, SHADOW_MAP_CAMERA_NEAR},
        resources::SceneResources,
    },
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub static DirectionalShadowMapFragmentShader: FragmentShaderFn =
    |context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        let p = Vec4::new(sample.world_pos, 1.0)
            * context.view_inverse_transform
            * context.projection_transform;

        let (near, far) = (
            context.projection_z_near.unwrap_or(SHADOW_MAP_CAMERA_NEAR),
            context
                .projection_z_far
                .unwrap_or(DEFAULT_SHADOW_MAP_CAMERA_FAR),
        );

        let _max_depth = far - near;

        let depth = p.z.max(near).min(far);

        if depth > 1.0 {
            Default::default()
        } else {
            let distance_alpha = depth / 100.0;

            Color::from_vec3(Vec3::ones() * distance_alpha)
        }
    };
