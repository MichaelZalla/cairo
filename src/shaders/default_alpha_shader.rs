use std::sync::RwLockReadGuard;

use crate::{
    shader::{alpha::AlphaShaderFn, ShaderContext},
    texture::sample::sample_nearest,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub static DEFAULT_ALPHA_SHADER: AlphaShaderFn =
    |context: &RwLockReadGuard<'_, ShaderContext>, out: &DefaultVertexOut| -> bool {
        // Check if this fragment can be discarded.

        match context.active_material {
            Some(material_raw_mut) => unsafe {
                match &(*material_raw_mut).alpha_map {
                    Some(texture) => {
                        // Read in a per-fragment normal, with components in the
                        // range [0, 255].

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

        true
    };
