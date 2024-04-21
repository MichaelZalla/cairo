use crate::{
    scene::resources::SceneResources,
    shader::{alpha::AlphaShaderFn, context::ShaderContext},
    texture::sample::sample_nearest,
    vertex::default_vertex_out::DefaultVertexOut,
};

pub static DEFAULT_ALPHA_SHADER: AlphaShaderFn =
    |context: &ShaderContext, resources: &SceneResources, out: &DefaultVertexOut| -> bool {
        // Check if this fragment can be discarded.

        match &context.active_material {
            Some(name) => {
                match resources.material.borrow().get(&name) {
                    Some(material) => {
                        match material.alpha_map {
                            Some(handle) => {
                                match resources.texture.borrow().get(&handle) {
                                    Ok(entry) => {
                                        let map = &entry.item;

                                        // Read in a per-fragment normal, with components in the
                                        // range [0, 255].

                                        let (r, _g, _b) = sample_nearest(out.uv, map, None);

                                        if r < 4 {
                                            return false;
                                        }
                                    }
                                    Err(err) => {
                                        panic!(
                                            "Failed to get TextureMap from Arena: {:?}: {}",
                                            name, err
                                        )
                                    }
                                }
                            }
                            None => (),
                        }
                    }
                    None => (),
                }
            }
            None => (),
        }

        true
    };
