use std::cell::RefCell;

use serde::{Deserialize, Serialize};

use crate::{
    entity::Entity,
    material::cache::MaterialCache,
    mesh::Mesh,
    resource::arena::Arena,
    serde::PostDeserialize,
    texture::{cubemap::CubeMap, map::TextureMap},
    vec::vec3::Vec3,
};

use super::{
    camera::Camera,
    environment::Environment,
    light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneResources {
    pub camera: RefCell<Arena<Camera>>,
    pub environment: RefCell<Arena<Environment>>,
    pub ambient_light: RefCell<Arena<AmbientLight>>,
    pub directional_light: RefCell<Arena<DirectionalLight>>,
    pub point_light: RefCell<Arena<PointLight>>,
    pub spot_light: RefCell<Arena<SpotLight>>,
    pub mesh: RefCell<Arena<Mesh>>,
    pub entity: RefCell<Arena<Entity>>,
    pub material: RefCell<MaterialCache>,
    pub texture_u8: RefCell<Arena<TextureMap>>,
    pub texture_vec3: RefCell<Arena<TextureMap<Vec3>>>,
    pub cubemap_u8: RefCell<Arena<CubeMap>>,
    pub cubemap_vec3: RefCell<Arena<CubeMap<Vec3>>>,
}

impl PostDeserialize for SceneResources {
    fn post_deserialize(&mut self) {
        self.camera.borrow_mut().post_deserialize();
        self.environment.borrow_mut().post_deserialize();
        self.ambient_light.borrow_mut().post_deserialize();
        self.directional_light.borrow_mut().post_deserialize();
        self.point_light.borrow_mut().post_deserialize();
        self.spot_light.borrow_mut().post_deserialize();
        self.mesh.borrow_mut().post_deserialize();
        self.entity.borrow_mut().post_deserialize();
        self.material.borrow_mut().post_deserialize();
        self.texture_u8.borrow_mut().post_deserialize();
        self.texture_vec3.borrow_mut().post_deserialize();
        self.cubemap_u8.borrow_mut().post_deserialize();
        self.cubemap_vec3.borrow_mut().post_deserialize();
    }
}

impl Default for SceneResources {
    fn default() -> Self {
        Self {
            camera: RefCell::new(Arena::<_>::new()),
            environment: RefCell::new(Arena::<_>::new()),
            ambient_light: RefCell::new(Arena::<_>::new()),
            directional_light: RefCell::new(Arena::<_>::new()),
            point_light: RefCell::new(Arena::<_>::new()),
            spot_light: RefCell::new(Arena::<_>::new()),
            mesh: RefCell::new(Arena::<_>::new()),
            entity: RefCell::new(Arena::<_>::new()),
            material: RefCell::new(Default::default()),
            texture_u8: RefCell::new(Arena::<_>::new()),
            texture_vec3: RefCell::new(Arena::<_>::new()),
            cubemap_u8: RefCell::new(Arena::<_>::new()),
            cubemap_vec3: RefCell::new(Arena::<_>::new()),
        }
    }
}
