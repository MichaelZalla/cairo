use crate::{
    matrix::Mat4,
    resource::handle::Handle,
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
    pub active_material: Option<String>,
    pub active_uv_test_texture_map: Option<Handle>,
    pub active_environment_map: Option<Handle>,
    pub ambient_light: Option<Handle>,
    pub directional_light: Option<Handle>,
    pub point_lights: Vec<Handle>,
    pub spot_lights: Vec<Handle>,
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
            active_material: None,
            active_uv_test_texture_map: None,
            active_environment_map: None,
            ambient_light: None,
            directional_light: None,
            point_lights: vec![],
            spot_lights: vec![],
        }
    }
}

impl ShaderContext {
    pub fn new(
        world_transform: Mat4,
        view_position: Vec4,
        view_inverse_transform: Mat4,
        projection_transform: Mat4,
        ambient_light: Option<Handle>,
        directional_light: Option<Handle>,
        point_lights: Vec<Handle>,
        spot_lights: Vec<Handle>,
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
        self.projection_transform
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

    pub fn set_ambient_light(&mut self, light: Option<Handle>) {
        self.ambient_light = light;
    }

    pub fn set_directional_light(&mut self, light: Option<Handle>) {
        self.directional_light = light;
    }

    pub fn get_point_lights(&self) -> &Vec<Handle> {
        &self.point_lights
    }

    pub fn get_point_lights_mut(&mut self) -> &mut Vec<Handle> {
        &mut self.point_lights
    }

    pub fn get_spot_lights(&self) -> &Vec<Handle> {
        &self.spot_lights
    }

    pub fn get_spot_lights_mut(&mut self) -> &mut Vec<Handle> {
        &mut self.spot_lights
    }

    pub fn set_active_material(&mut self, optional_handle: Option<String>) {
        self.active_material = optional_handle;
    }

    pub fn set_active_uv_test_texture_map(&mut self, optional_handle: Option<Handle>) {
        self.active_uv_test_texture_map = optional_handle;
    }

    pub fn set_active_environment_map(&mut self, optional_handle: Option<Handle>) {
        self.active_environment_map = optional_handle;
    }
}
