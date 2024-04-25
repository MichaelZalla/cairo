use std::fmt;

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
    pub ambient_color: Vec3,
    pub ambient_color_map: Option<Handle>,
    pub ambient_occlusion_map: Option<Handle>,
    pub diffuse_color: Vec3,
    pub diffuse_color_map: Option<Handle>,
    pub specular_color: Vec3,
    pub specular_exponent: i32, // aka "shininess"
    pub specular_exponent_map: Option<Handle>,
    pub emissive_color: Vec3,
    pub emissive_color_map: Option<Handle>,
    pub dissolve: f32,
    pub transparency: f32,
    pub alpha_map: Option<Handle>,
    pub translucency: Vec3,
    pub index_of_refraction: f32,
    pub normal_map: Option<Handle>,
    pub displacement_map: Option<Handle>,
    pub displacement_scale: f32,
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
            &mut self.ambient_color_map,
            &mut self.ambient_occlusion_map,
            &mut self.diffuse_color_map,
            &mut self.specular_exponent_map,
            &mut self.emissive_color_map,
            &mut self.normal_map,
            &mut self.displacement_map,
            &mut self.alpha_map,
        ];

        for handle in optional_handles {
            match handle {
                Some(handle) => match texture_arena.get_mut(handle) {
                    Ok(entry) => {
                        let map = &mut entry.item;

                        if !map.is_loaded {
                            map.load(rendering_context)?;
                        }
                    }
                    Err(_err) => panic!("Invalid TextureMap handle!"),
                },
                None => (),
            }
        }

        Ok(())
    }
}

impl fmt::Display for Material {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Material (\"{}\")", self.name)?;

        match &self.material_source {
            Some(source) => {
                writeln!(v, "  > Material source : {}", source)?;
            }
            None => (),
        }

        writeln!(v, "  > Illumination model: {}", self.illumination_model)?;

        writeln!(
            v,
            "  > Ambient color: {}",
            color::Color::from_vec3(self.ambient_color * 255.0)
        )?;

        match &self.ambient_color_map {
            Some(handle) => {
                writeln!(v, "  > Ambient color map: {}", handle.uuid)?;
            }
            None => (),
        }

        match &self.ambient_occlusion_map {
            Some(handle) => {
                writeln!(v, "  > Ambient occlusion map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Diffuse color: {}",
            color::Color::from_vec3(self.diffuse_color * 255.0)
        )?;

        match &self.diffuse_color_map {
            Some(handle) => {
                writeln!(v, "  > Diffuse color map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Specular color: {}",
            color::Color::from_vec3(self.specular_color * 255.0)
        )?;

        writeln!(v, "  > Specular exponent: {}", self.specular_exponent)?;

        match &self.specular_exponent_map {
            Some(handle) => {
                writeln!(v, "  > Specular exponent map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Emissive color: {}",
            color::Color::from_vec3(self.emissive_color * 255.0)
        )?;

        match &self.emissive_color_map {
            Some(handle) => {
                writeln!(v, "  > Emissive color map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(v, "  > Dissolve: {}", self.dissolve)?;

        writeln!(v, "  > Transparency: {}", self.transparency)?;

        match &self.alpha_map {
            Some(handle) => {
                writeln!(v, "  > Alpha map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(v, "  > Translucency: {}", self.translucency)?;

        writeln!(v, "  > Index of refraction: {}", self.index_of_refraction)?;

        match &self.normal_map {
            Some(handle) => writeln!(v, "  > Normal map: {}", handle.uuid),
            _ => Ok(()),
        }?;

        writeln!(v, "  > Displacement scale: {}", self.displacement_scale)?;

        match &self.displacement_map {
            Some(handle) => writeln!(v, "  > Displacement map: {}", handle.uuid),
            _ => Ok(()),
        }
    }
}
