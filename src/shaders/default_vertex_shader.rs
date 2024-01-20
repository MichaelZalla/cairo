use std::sync::RwLockReadGuard;

use crate::{
    matrix::Mat4,
    shader::{vertex::VertexShaderFn, ShaderContext},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub static DEFAULT_VERTEX_SHADER: VertexShaderFn =
    |context: &RwLockReadGuard<'_, ShaderContext>, v: &DefaultVertexIn| -> DefaultVertexOut {
        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut::new();

        out.position = Vec4::new(v.position, 1.0) * context.world_view_projection_transform;

        // debug_assert!(out.position.w != 0.0);

        let world_pos = Vec4::new(v.position, 1.0) * context.world_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        // Compute a tangent-space to world-space transform.

        let normal = (Vec4::new(v.normal, 0.0) * context.world_transform).as_normal();
        let tangent = (Vec4::new(v.tangent, 0.0) * context.world_transform).as_normal();
        let bitangent = (Vec4::new(v.bitangent, 0.0) * context.world_transform).as_normal();

        out.normal = normal;

        let (t, b, n) = (tangent, bitangent, normal);

        // @NOTE(mzalla) Reversed Z-axis for our renderer's coordinate system.

        out.tbn = Mat4::new_from_elements([
            [t.x, t.y, t.z, 0.0],
            [b.x, b.y, b.z, 0.0],
            [n.x, n.y, n.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        out.color = v.color.clone();

        out.uv = v.uv.clone();

        out
    };
