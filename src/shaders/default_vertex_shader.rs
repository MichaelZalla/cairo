use std::sync::RwLockReadGuard;

use crate::{
    shader::{vertex::VertexShaderFn, ShaderContext},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub static DEFAULT_VERTEX_SHADER: VertexShaderFn =
    |context: &RwLockReadGuard<'_, ShaderContext>, v: &DefaultVertexIn| -> DefaultVertexOut {
        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut::new();

        out.position = Vec4::new(v.position, 1.0) * context.world_view_projection_transform;

        debug_assert!(out.position.w != 0.0);

        let world_pos = Vec4::new(v.position, 1.0) * context.world_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        out.normal = Vec4::new(v.normal, 0.0) * context.world_transform;
        out.normal = out.normal.as_normal();

        out.color = v.color.clone();

        out.uv = v.uv.clone();

        out
    };
