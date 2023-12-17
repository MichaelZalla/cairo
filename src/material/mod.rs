use crate::vec::vec3::Vec3;

#[derive(Debug, Clone, Default)]
pub struct TextureMap {
    pub filepath: String,
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u8>,
}

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
