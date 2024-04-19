use crate::{
    shader::{alpha::AlphaShaderFn, context::ShaderContext},
    texture::sample::sample_nearest,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub static DEFAULT_ALPHA_SHADER: AlphaShaderFn =
    |context: &ShaderContext, out: &DefaultVertexOut| -> bool {
        // Check if this fragment can be discarded.

        match context.active_material {
            Some(material_raw_mut) => unsafe {
                match &(*material_raw_mut).alpha_map {
                    Some(texture_handle) => {
                        match &context.texture_arena {
                            Some(arena) => match arena.get(texture_handle) {
                                Ok(entry) => {
                                    let map = &entry.item;

                                    // Read in a per-fragment normal, with components in the
                                    // range [0, 255].

                                    let (r, _g, _b) = sample_nearest(out.uv, map, None);

                                    if r < 4 {
                                        return false;
                                    }
                                }
                                Err(_) => panic!("Invalid TextureMap handle!"),
                            },
                            None => (),
                        }
                    }
                    None => (),
                }
            },
            None => (),
        }

        true
    };
