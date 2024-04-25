use serde::{Deserialize, Serialize};

use crate::{
    app::context::ApplicationRenderingContext,
    color,
    resource::{arena::Arena, handle::Handle},
    serde::PostDeserialize,
    texture::map::TextureMap,
    vec::vec3::Vec3,
};

pub mod cache;
pub mod mtl;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub material_source: Option<String>,
    pub illumination_model: u8,
    // Common attributes
    pub emissive_color: Vec3,
    pub emissive_color_map: Option<Handle>,
    pub dissolve: f32,
    pub alpha_map: Option<Handle>,
    pub transparency: f32,
    pub transparency_map: Option<Handle>,
    pub translucency: Vec3,
    pub index_of_refraction: f32,
    pub normal_map: Option<Handle>,
    pub displacement_map: Option<Handle>,
    pub displacement_scale: f32,
    pub ambient_occlusion_map: Option<Handle>,
    // Blinn-Phong attributes
    pub ambient_color: Vec3,
    pub ambient_color_map: Option<Handle>,
    pub diffuse_color: Vec3,
    pub diffuse_color_map: Option<Handle>,
    pub specular_color: Vec3,
    pub specular_color_map: Option<Handle>,
    pub specular_exponent: i32, // aka "shininess"
    pub specular_exponent_map: Option<Handle>,
}

impl PostDeserialize for Material {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl Material {
    pub fn new(name: String) -> Self {
        Material {
            name,
            diffuse_color: color::WHITE.to_vec3() / 255.0,
            specular_exponent: 8,
            ..Default::default()
        }
    }

    pub fn load_all_maps(
        &mut self,
        texture_arena: &mut Arena<TextureMap>,
        rendering_context: &ApplicationRenderingContext,
    ) -> Result<(), String> {
        let optional_handles = [
            &mut self.alpha_map,
            &mut self.ambient_color_map,
            &mut self.ambient_occlusion_map,
            &mut self.diffuse_color_map,
            &mut self.displacement_map,
            &mut self.emissive_color_map,
            &mut self.normal_map,
            &mut self.specular_color_map,
            &mut self.specular_exponent_map,
            &mut self.transparency_map,
        ];

        optional_handles.into_iter().for_each(|optional_handle| {
            if let Some(handle) = optional_handle {
                match texture_arena.get_mut(handle) {
                    Ok(entry) => {
                        let map = &mut entry.item;

                        if !map.is_loaded {
                            map.load(rendering_context).unwrap();
                        }
                    }
                    Err(_err) => panic!("Invalid TextureMap handle!"),
                }
            }
        });

        Ok(())
    }
}
