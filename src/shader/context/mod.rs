use crate::{
    material::Material,
    matrix::Mat4,
    scene::light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
    texture::{cubemap::CubeMap, map::TextureMap},
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct ShaderContext {
    pub world_transform: Mat4,
    pub view_position: Vec4,
    pub view_inverse_transform: Mat4,
    pub world_view_transform: Mat4,
    pub projection_transform: Mat4,
    pub world_view_projection_transform: Mat4,
    pub default_specular_exponent: i32,
    pub active_material: Option<*const Material>,
    pub active_uv_test_texture_map: Option<*const TextureMap>,
    pub active_environment_map: Option<*const CubeMap>,
    pub ambient_light: AmbientLight,
    pub directional_light: DirectionalLight,
    pub point_lights: Vec<PointLight>,
    pub spot_lights: Vec<SpotLight>,
}

impl Default for ShaderContext {
    fn default() -> Self {
        Self {
            world_transform: Mat4::identity(),
            view_position: Default::default(),
            view_inverse_transform: Mat4::identity(),
            world_view_transform: Mat4::identity(),
            projection_transform: Mat4::identity(),
            world_view_projection_transform: Default::default(),
            default_specular_exponent: 8,
            active_material: Default::default(),
            active_uv_test_texture_map: None,
            active_environment_map: None,
            ambient_light: Default::default(),
            directional_light: Default::default(),
            point_lights: Default::default(),
            spot_lights: Default::default(),
        }
    }
}

impl ShaderContext {
    pub fn new(
        world_transform: Mat4,
        view_position: Vec4,
        view_inverse_transform: Mat4,
        projection_transform: Mat4,
        ambient_light: AmbientLight,
        directional_light: DirectionalLight,
        point_lights: Vec<PointLight>,
        spot_lights: Vec<SpotLight>,
    ) -> Self {
        Self {
            world_transform,
            view_position,
            view_inverse_transform,
            projection_transform,
            world_view_transform: world_transform * view_inverse_transform,
            world_view_projection_transform: world_transform
                * view_inverse_transform
                * projection_transform,
            default_specular_exponent: 8,
            active_material: None,
            active_uv_test_texture_map: None,
            active_environment_map: None,
            ambient_light,
            directional_light,
            point_lights,
            spot_lights,
        }
    }

    pub fn get_world_transform(&mut self) -> Mat4 {
        self.world_transform
    }

    pub fn set_world_transform(&mut self, mat: Mat4) {
        self.world_transform = mat;

        self.world_view_transform = self.world_transform * self.view_inverse_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_view_position(&mut self, position: Vec4) {
        self.view_position = position;
    }

    pub fn set_view_inverse_transform(&mut self, mat: Mat4) {
        self.view_inverse_transform = mat;

        self.world_view_transform = self.world_transform * self.view_inverse_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn get_projection(&self) -> Mat4 {
        return self.projection_transform;
    }

    pub fn set_projection(&mut self, projection_transform: Mat4) {
        self.projection_transform = projection_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn to_ndc_space(&self, world_space_position: Vec3) -> Vec3 {
        let ndc_space_position = {
            let mut view_projection_space_position = Vec4::new(world_space_position, 1.0)
                * self.view_inverse_transform
                * self.projection_transform;

            let w_inverse = 1.0 / view_projection_space_position.w;

            view_projection_space_position *= w_inverse;

            view_projection_space_position.x = (view_projection_space_position.x + 1.0) / 2.0;
            view_projection_space_position.y = (-view_projection_space_position.y + 1.0) / 2.0;

            view_projection_space_position
        };

        ndc_space_position.to_vec3()
    }

    pub fn set_ambient_light(&mut self, light: AmbientLight) {
        self.ambient_light = light;
    }

    pub fn set_directional_light(&mut self, light: DirectionalLight) {
        self.directional_light = light;
    }

    pub fn set_point_light(&mut self, index: usize, light: PointLight) {
        if index > self.point_lights.len() {
            panic!("Called ShaderContext.set_point_light() with an index greater than point_lights.len()!");
        } else if index == self.point_lights.len() {
            self.point_lights.push(light);
        } else {
            self.point_lights[index] = light;
        }
    }

    pub fn set_spot_light(&mut self, index: usize, light: SpotLight) {
        if index > self.point_lights.len() {
            panic!("Called ShaderContext.set_spot_light() with an index greater than spot_lights.len()!");
        } else if index == self.spot_lights.len() {
            self.spot_lights.push(light);
        } else {
            self.spot_lights[index] = light;
        }
    }

    pub fn set_active_material(&mut self, material_option: Option<*const Material>) {
        match material_option {
            Some(material_raw_mut) => {
                self.active_material = Some(material_raw_mut);
            }
            None => {
                self.active_material = None;
            }
        }
    }

    pub fn set_active_uv_test_texture_map(&mut self, map: Option<*const TextureMap>) {
        match map {
            Some(texture_raw_mut) => {
                self.active_uv_test_texture_map = Some(texture_raw_mut);
            }
            None => {
                self.active_uv_test_texture_map = None;
            }
        }
    }

    pub fn set_active_environment_map(&mut self, skybox: Option<*const CubeMap>) {
        match skybox {
            Some(cubemap_raw_mut) => {
                self.active_environment_map = Some(cubemap_raw_mut);
            }
            None => {
                self.active_environment_map = None;
            }
        }
    }
}