use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    entity::Entity,
    material::Material,
    mesh::Mesh,
    resource::arena::Arena,
    serde::PostDeserialize,
    texture::{cubemap::CubeMap, map::TextureMap},
    vec::{vec2::Vec2, vec3::Vec3},
};

use super::{
    camera::Camera,
    environment::Environment,
    light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
    skybox::Skybox,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SceneResources {
    pub camera: Rc<RefCell<Arena<Camera>>>,
    pub environment: Rc<RefCell<Arena<Environment>>>,
    pub skybox: Rc<RefCell<Arena<Skybox>>>,
    pub ambient_light: Rc<RefCell<Arena<AmbientLight>>>,
    pub directional_light: Rc<RefCell<Arena<DirectionalLight>>>,
    pub point_light: Rc<RefCell<Arena<PointLight>>>,
    pub spot_light: Rc<RefCell<Arena<SpotLight>>>,
    pub mesh: Rc<RefCell<Arena<Mesh>>>,
    pub entity: Rc<RefCell<Arena<Entity>>>,
    pub material: Rc<RefCell<Arena<Material>>>,
    pub texture_u8: Rc<RefCell<Arena<TextureMap>>>,
    pub texture_vec2: Rc<RefCell<Arena<TextureMap<Vec2>>>>,
    pub texture_vec3: Rc<RefCell<Arena<TextureMap<Vec3>>>>,
    pub cubemap_u8: Rc<RefCell<Arena<CubeMap>>>,
    pub cubemap_f32: Rc<RefCell<Arena<CubeMap<f32>>>>,
    pub cubemap_vec3: Rc<RefCell<Arena<CubeMap<Vec3>>>>,
}

impl PostDeserialize for SceneResources {
    fn post_deserialize(&mut self) {
        self.camera.borrow_mut().post_deserialize();
        self.environment.borrow_mut().post_deserialize();
        self.skybox.borrow_mut().post_deserialize();
        self.ambient_light.borrow_mut().post_deserialize();
        self.directional_light.borrow_mut().post_deserialize();
        self.point_light.borrow_mut().post_deserialize();
        self.spot_light.borrow_mut().post_deserialize();
        self.mesh.borrow_mut().post_deserialize();
        self.entity.borrow_mut().post_deserialize();
        self.material.borrow_mut().post_deserialize();
        self.texture_u8.borrow_mut().post_deserialize();
        self.texture_vec2.borrow_mut().post_deserialize();
        self.texture_vec3.borrow_mut().post_deserialize();
        self.cubemap_u8.borrow_mut().post_deserialize();
        self.cubemap_f32.borrow_mut().post_deserialize();
        self.cubemap_vec3.borrow_mut().post_deserialize();
    }
}
