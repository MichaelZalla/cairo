use crate::{image::TextureMap, vec::vec3::Vec3};

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub diffuse: Vec3,
    pub diffuse_map: Option<TextureMap>,
    pub normal_map: Option<TextureMap>,
}

impl Material {
    pub fn new() -> Self {
        return Default::default();
    }
}
