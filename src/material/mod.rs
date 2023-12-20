use std::fmt;

use crate::{color, image::TextureMap, vec::vec3::Vec3};

pub mod mtl;

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub illumination_model: u8,
    pub ambient_color: Vec3,
    pub ambient_map: Option<TextureMap>,
    pub diffuse_color: Vec3,
    pub diffuse_map: Option<TextureMap>,
    pub specular_color: Vec3,
    pub specular_exponent: f32,
    pub emissive_color: Vec3,
    pub dissolve: f32,
    pub transparency: f32,
    pub transmission_filter_color: Vec3,
    pub index_of_refraction: f32,
    pub normal_map: Option<TextureMap>,
}

impl Material {
    pub fn new() -> Self {
        return Default::default();
    }
}

impl fmt::Display for Material {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(v, "Material")?;

        writeln!(v, "  > Illumination model: {}", self.illumination_model)?;

        writeln!(
            v,
            "  > Ambient color: {}",
            color::Color::from_vec3(self.ambient_color)
        )?;

        match &self.ambient_map {
            Some(map) => {
                writeln!(v, "  > Ambient map: {}", map.info.filepath)?;
            }
            None => (),
        }

        writeln!(
            v,
            "  > Diffuse color: {}",
            color::Color::from_vec3(self.diffuse_color)
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
            color::Color::from_vec3(self.specular_color)
        )?;

        writeln!(v, "  > Specular exponent: {}", self.specular_exponent)?;

        writeln!(
            v,
            "  > Emissive color: {}",
            color::Color::from_vec3(self.emissive_color)
        )?;

        writeln!(v, "  > Dissolve: {}", self.dissolve)?;

        writeln!(v, "  > Transparency: {}", self.transparency)?;

        writeln!(
            v,
            "  > Transmission filter color: {}",
            self.transmission_filter_color
        )?;

        writeln!(v, "  > Index of refraction: {}", self.index_of_refraction)?;

        match &self.normal_map {
            Some(map) => writeln!(v, "  > Normal map: {}", map.info.filepath),
            _ => Ok(()),
        }
    }
}
