use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    app::context::ApplicationRenderingContext,
    color,
    resource::{arena::Arena, handle::Handle},
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
    pub ambient_map: Option<Handle>,
    pub ambient_occlusion_map: Option<Handle>,
    pub diffuse_color: Vec3,
    pub diffuse_map: Option<Handle>,
    pub specular_color: Vec3,
    pub specular_exponent: i32, // aka "shininess"
    pub specular_map: Option<Handle>,
    pub emissive_color: Vec3,
    pub emissive_map: Option<Handle>,
    pub dissolve: f32,
    pub transparency: f32,
    pub alpha_map: Option<Handle>,
    pub transmission_filter_color: Vec3,
    pub index_of_refraction: f32,
    pub normal_map: Option<Handle>,
    pub displacement_map: Option<Handle>,
    pub displacement_scale: f32,
}

impl Material {
    pub fn new(name: String) -> Self {
        let mut mat: Material = Default::default();

        mat.name = name;
        mat.diffuse_color = color::WHITE.to_vec3() / 255.0;
        mat.specular_exponent = 8;

        mat
    }

    pub fn load_all_maps(
        &mut self,
        texture_arena: &mut Arena<TextureMap>,
        rendering_context: &ApplicationRenderingContext,
    ) -> Result<(), String> {
        let optional_handles = [
            &mut self.ambient_map,
            &mut self.ambient_occlusion_map,
            &mut self.diffuse_map,
            &mut self.specular_map,
            &mut self.emissive_map,
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

        match &self.ambient_map {
            Some(handle) => {
                writeln!(v, "  > Ambient map: {}", handle.uuid)?;
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

        match &self.diffuse_map {
            Some(handle) => {
                writeln!(v, "  > Diffuse map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Specular color: {}",
            color::Color::from_vec3(self.specular_color * 255.0)
        )?;

        writeln!(v, "  > Specular exponent: {}", self.specular_exponent)?;

        match &self.specular_map {
            Some(handle) => {
                writeln!(v, "  > Specular map: {}", handle.uuid)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Emissive color: {}",
            color::Color::from_vec3(self.emissive_color * 255.0)
        )?;

        match &self.emissive_map {
            Some(handle) => {
                writeln!(v, "  > Emissive map: {}", handle.uuid)?;
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

        writeln!(
            v,
            "  > Transmission filter color: {}",
            self.transmission_filter_color
        )?;

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
