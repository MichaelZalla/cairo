use std::sync::RwLock;

use crate::{
    shader::{alpha::AlphaShader, ShaderContext},
    texture::sample::sample_nearest,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub struct DefaultAlphaShader<'a> {
    context: &'a RwLock<ShaderContext>,
}

impl<'a> DefaultAlphaShader<'a> {
    pub fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }
}

impl<'a> AlphaShader<'a> for DefaultAlphaShader<'a> {
    fn new(context: &'a RwLock<ShaderContext>) -> Self {
        Self { context }
    }

    fn call(&self, out: &DefaultVertexOut) -> bool {
        let context = self.context.read().unwrap();

        // Check if this fragment can be discarded

        match context.active_material {
            Some(mat_raw_mut) => unsafe {
                match &(*mat_raw_mut).alpha_map {
                    Some(texture) => {
                        // Read in a per-fragment normal, with components in the range [0, 255].
                        let (r, _g, _b) = sample_nearest(out.uv, texture, None);

                        if r < 4 {
                            return false;
                        }
                    }
                    None => (),
                }
            },
            None => (),
        }

        return true;
    }
}
