#![allow(non_upper_case_globals)]

use crate::{
    color::Color,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec4::Vec4,
};

pub const NormalFragmentShader: FragmentShaderFn =
    |_context: &ShaderContext, _resources: &SceneResources, sample: &GeometrySample| -> Color {
        // let context: std::sync::RefCell<ShaderContext> = self.context.read().unwrap();

        // Emit only the world-space normal (RBG space) for this fragment.

        let world_space_surface_normal = Vec4::new(sample.normal, 1.0);

        // let view_space_surface_normal =
        //     (world_space_surface_normal * context.view_inverse_transform).as_normal();

        Color {
            r: world_space_surface_normal.x,
            g: world_space_surface_normal.y,
            b: (1.0 - world_space_surface_normal.z),
            a: 1.0,
        }
    };
