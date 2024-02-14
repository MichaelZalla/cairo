use std::fmt;

use crate::{
    color, context::ApplicationRenderingContext, mesh::MaterialSource, texture::map::TextureMap,
    vec::vec3::Vec3,
};

pub mod cache;
pub mod mtl;

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub name: String,
    pub material_source: Option<MaterialSource>,
    pub illumination_model: u8,
    pub ambient_color: Vec3,
    pub ambient_map: Option<TextureMap>,
    pub ambient_occlusion_map: Option<TextureMap>,
    pub diffuse_color: Vec3,
    pub diffuse_map: Option<TextureMap>,
    pub specular_color: Vec3,
    pub specular_exponent: i32, // aka "shininess"
    pub specular_map: Option<TextureMap>,
    pub emissive_color: Vec3,
    pub emissive_map: Option<TextureMap>,
    pub dissolve: f32,
    pub transparency: f32,
    pub alpha_map: Option<TextureMap>,
    pub transmission_filter_color: Vec3,
    pub index_of_refraction: f32,
    pub normal_map: Option<TextureMap>,
    pub displacement_map: Option<TextureMap>,
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
        rendering_context: &ApplicationRenderingContext,
    ) -> Result<(), String> {
        // Ambient map
        match &mut self.ambient_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Ambient occlusion map
        match &mut self.ambient_occlusion_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Diffuse map
        match &mut self.diffuse_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Specular map
        match &mut self.specular_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Emissive map
        match &mut self.emissive_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Normal map
        match &mut self.normal_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Displacement map
        match &mut self.displacement_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        // Alpha map
        match &mut self.alpha_map {
            Some(map) => {
                if !map.is_loaded {
                    map.load(rendering_context)?
                } else {
                }
            }
            None => (),
        }

        Ok(())
    }
}

impl fmt::Display for Material {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Material (\"{}\")", self.name)?;

        match &self.material_source {
            Some(source) => {
                writeln!(v, "  > Material source : {}", source.filepath)?;
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
            Some(map) => {
                writeln!(v, "  > Ambient map: {}", map.info.filepath)?;
            }
            None => (),
        }

        match &self.ambient_occlusion_map {
            Some(map) => {
                writeln!(v, "  > Ambient occlusion map: {}", map.info.filepath)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Diffuse color: {}",
            color::Color::from_vec3(self.diffuse_color * 255.0)
        )?;

        match &self.diffuse_map {
            Some(map) => {
                writeln!(v, "  > Diffuse map: {}", map.info.filepath)?;
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
            Some(map) => {
                writeln!(v, "  > Specular map: {}", map.info.filepath)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Emissive color: {}",
            color::Color::from_vec3(self.emissive_color * 255.0)
        )?;

        match &self.emissive_map {
            Some(map) => {
                writeln!(v, "  > Emissive map: {}", map.info.filepath)?;
            }
            None => (),
        }

        writeln!(v, "  > Dissolve: {}", self.dissolve)?;

        writeln!(v, "  > Transparency: {}", self.transparency)?;

        match &self.alpha_map {
            Some(map) => {
                writeln!(v, "  > Alpha map: {}", map.info.filepath)?;
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
            Some(map) => writeln!(v, "  > Normal map: {}", map.info.filepath),
            _ => Ok(()),
        }?;

        match &self.displacement_map {
            Some(map) => writeln!(v, "  > Displacement map: {}", map.info.filepath),
            _ => Ok(()),
        }
    }
}
