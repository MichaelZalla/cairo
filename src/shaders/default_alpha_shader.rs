use crate::{
    scene::resources::SceneResources,
    shader::{alpha::AlphaShaderFn, context::ShaderContext},
    texture::sample::sample_nearest_u8,
    vertex::default_vertex_out::DefaultVertexOut,
};

#[allow(non_upper_case_globals)]
pub static DefaultAlphaShader: AlphaShaderFn =
    |context: &ShaderContext, resources: &SceneResources, out: &DefaultVertexOut| -> bool {
        // Check if this fragment can be discarded.

        if let Some(material_handle) = &context.active_material
            && let Ok(entry) = resources.material.borrow().get(material_handle)
        {
            let material = &entry.item;

            if let Some(alpha_map_handle) = material.alpha_map {
                match resources.texture_u8.borrow().get(&alpha_map_handle) {
                    Ok(entry) => {
                        let map = &entry.item;

                        // Read in a per-fragment normal, with components in the
                        // range [0, 255].

                        let (r, _g, _b) = sample_nearest_u8(out.uv, map, None);

                        if r < 4 {
                            return false;
                        }
                    }
                    Err(err) => {
                        panic!(
                            "Failed to get TextureMap from Arena: {:?}: {}",
                            material_handle, err
                        )
                    }
                }
            }
        }

        true
    };
