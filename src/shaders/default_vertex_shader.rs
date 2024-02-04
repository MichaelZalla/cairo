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

        out.p = Vec4::new(v.p, 1.0) * context.world_view_projection_transform;

        debug_assert!(out.p.w != 0.0);

        let world_pos = Vec4::new(v.p, 1.0) * context.world_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        out.n = Vec4::new(v.n, 0.0) * context.world_transform;
        out.n = out.n.as_normal();

        out.c = v.c.clone();

        out.uv = v.uv.clone();

        out
    };
