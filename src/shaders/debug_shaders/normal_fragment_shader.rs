#![allow(non_upper_case_globals)]

use std::sync::{RwLock, RwLockReadGuard};

use crate::{
    color::Color,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
    },
    vec::vec4::Vec4,
};

pub struct NormalFragmentShader<'a> {
    context: &'a RwLock<ShaderContext>,
}

pub const NormalFragmentShader: FragmentShaderFn =
    |_context: &RwLockReadGuard<ShaderContext>, sample: &GeometrySample| -> Color {
        // let context: std::sync::RwLockReadGuard<'_, ShaderContext> = self.context.read().unwrap();

        // Emit only the world-space normal (RBG space) for this fragment.

        let world_space_surface_normal = Vec4::new(sample.normal, 1.0);

        // let view_space_surface_normal =
        //     (world_space_surface_normal * context.view_inverse_transform).as_normal();

        return Color {
            r: world_space_surface_normal.x,
            g: world_space_surface_normal.y,
            b: (1.0 - world_space_surface_normal.z),
            a: 1.0,
        };
    };
