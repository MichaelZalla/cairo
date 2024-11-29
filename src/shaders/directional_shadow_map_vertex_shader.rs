#![allow(non_upper_case_globals)]

use crate::{
    shader::{context::ShaderContext, vertex::VertexShaderFn},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub static DirectionalShadowMapVertexShader: VertexShaderFn =
    |context: &ShaderContext, v: &DefaultVertexIn| -> DefaultVertexOut {
        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut {
            position_projection_space: Vec4::new(v.position, 1.0)
                * context.world_view_projection_transform,
            ..Default::default()
        };

        let world_pos = Vec4::new(v.position, 1.0) * context.world_transform;

        out.position_world_space = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        out
    };
