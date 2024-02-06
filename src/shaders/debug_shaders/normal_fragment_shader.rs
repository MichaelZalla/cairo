#![allow(non_upper_case_globals)]

use std::sync::{RwLock, RwLockReadGuard};

use crate::{
    color::Color,
    shader::{fragment::FragmentShaderFn, geometry::sample::GeometrySample, ShaderContext},
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
            r: (world_space_surface_normal.x * 255.0) as u8,
            g: (world_space_surface_normal.y * 255.0) as u8,
            b: ((1.0 - world_space_surface_normal.z) * 255.0) as u8,
            a: 255 as u8,
        };
    };
