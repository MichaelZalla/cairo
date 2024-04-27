use serde::{Deserialize, Serialize};

use crate::{
    entity::Entity,
    material::cache::MaterialCache,
    mesh::Mesh,
    resource::arena::Arena,
    serde::PostDeserialize,
    texture::{cubemap::CubeMap, map::TextureMap},
};

use super::{
    camera::Camera,
    environment::Environment,
    light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneResources {
    pub camera: Arena<Camera>,
    pub environment: Arena<Environment>,
    pub ambient_light: Arena<AmbientLight>,
    pub directional_light: Arena<DirectionalLight>,
    pub point_light: Arena<PointLight>,
    pub spot_light: Arena<SpotLight>,
    pub mesh: Arena<Mesh>,
    pub entity: Arena<Entity>,
    pub material: MaterialCache,
    pub texture: Arena<TextureMap>,
    pub skybox: Arena<CubeMap>,
}

impl PostDeserialize for SceneResources {
    fn post_deserialize(&mut self) {
        self.camera.post_deserialize();
        self.environment.post_deserialize();
        self.ambient_light.post_deserialize();
        self.directional_light.post_deserialize();
        self.point_light.post_deserialize();
        self.spot_light.post_deserialize();
        self.mesh.post_deserialize();
        self.entity.post_deserialize();
        self.material.post_deserialize();
        self.texture.post_deserialize();
        self.skybox.post_deserialize();
    }
}
