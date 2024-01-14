use std::sync::RwLock;

use crate::{
    shader::{vertex::VertexShader, ShaderContext},
    vec::{vec3::Vec3, vec4::Vec4},
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

pub struct DefaultVertexShader<'a> {
    pub context: &'a RwLock<ShaderContext>,
}

impl<'a> VertexShader<'a> for DefaultVertexShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }

    fn call(&self, v: &DefaultVertexIn) -> DefaultVertexOut {
        let context = self.context.read().unwrap();

        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut::new();

        out.p = Vec4::new(v.p, 1.0) * context.world_view_projection_transform;

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

        return out;
    }
}
