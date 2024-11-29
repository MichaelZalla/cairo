use crate::{
    matrix::Mat4,
    shader::{context::ShaderContext, vertex::VertexShaderFn},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{
        default_vertex_in::DefaultVertexIn,
        default_vertex_out::{DefaultVertexOut, TangentSpaceInfo},
    },
};

pub static DEFAULT_VERTEX_SHADER: VertexShaderFn = |context: &ShaderContext,
                                                    v: &DefaultVertexIn|
 -> DefaultVertexOut {
    let mut out = DefaultVertexOut::default();

    // World-space position.

    let position_world_space = Vec4::new(v.position, 1.0) * context.world_transform;

    out.position_world_space = Vec3 {
        x: position_world_space.x,
        y: position_world_space.y,
        z: position_world_space.z,
    };

    // View-space position.

    out.position_view_space = (Vec4::new(v.position, 1.0) * context.world_view_transform).to_vec3();

    // Projection-space position.

    out.position_projection_space =
        Vec4::new(v.position, 1.0) * context.world_view_projection_transform;

    // debug_assert!(out.position_projection_space.w != 0.0);

    // Compute a tangent-space to world-space transform.

    let normal_world_space = (v.normal * context.world_transform).as_normal();
    let tangent_world_space = (v.tangent * context.world_transform).as_normal();
    let bitangent_world_space = (v.bitangent * context.world_transform).as_normal();

    out.normal_world_space = normal_world_space;
    out.tangent_world_space = tangent_world_space;
    out.bitangent_world_space = bitangent_world_space;

    let (t, b, n) = (
        tangent_world_space,
        bitangent_world_space,
        normal_world_space,
    );

    let tbn = Mat4::tbn(t, b, n);

    let tbn_inverse = tbn.transposed();

    out.tangent_space_info = TangentSpaceInfo {
        tbn,
        tbn_inverse,
        normal: (normal_world_space * tbn_inverse),
        view_position: (context.view_position * tbn_inverse).to_vec3(),
        fragment_position: (position_world_space * tbn_inverse).to_vec3(),
    };

    out.color = v.color;
    out.uv = v.uv;

    out
};
