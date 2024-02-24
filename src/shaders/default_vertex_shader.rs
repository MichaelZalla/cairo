use crate::{
    matrix::Mat4,
    shader::{context::ShaderContext, vertex::VertexShaderFn},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{
        default_vertex_in::DefaultVertexIn,
        default_vertex_out::{DefaultVertexOut, TangentSpaceInfo},
    },
};

pub static DEFAULT_VERTEX_SHADER: VertexShaderFn =
    |context: &ShaderContext, v: &DefaultVertexIn| -> DefaultVertexOut {
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

        let tbn = Mat4::tbn(t.to_vec3(), b.to_vec3(), n.to_vec3());

        let tbn_inverse = tbn.transposed();

        out.tangent_space_info = TangentSpaceInfo {
            tbn,
            tbn_inverse,
            normal: (normal * tbn_inverse).to_vec3(),
            point_light_position: (Vec4::new(context.point_lights[0].position, 1.0) * tbn_inverse)
                .to_vec3(),
            view_position: (context.view_position * tbn_inverse).to_vec3(),
            fragment_position: (world_pos * tbn_inverse).to_vec3(),
        };

        out.color = v.color.clone();

        out.uv = v.uv.clone();

        out
    };
