use crate::{
    matrix::Mat4,
    resource::handle::Handle,
    vec::{vec3::Vec3, vec4::Vec4},
};

#[derive(Debug, Clone)]
pub struct ShaderContext {
    pub world_transform: Mat4,
    pub view_position: Vec4,
    pub view_inverse_transform: Mat4,
    pub world_view_transform: Mat4,
    pub projection_z_near: Option<f32>,
    pub projection_z_far: Option<f32>,
    pub projection_transform: Mat4,
    pub world_view_projection_transform: Mat4,
    pub active_material: Option<Handle>,
    pub active_uv_test_texture_map: Option<Handle>,
    pub active_hdr_map: Option<Handle>,
    pub ambient_radiance_map: Option<Handle>,
    pub ambient_diffuse_irradiance_map: Option<Handle>,
    pub ambient_specular_prefiltered_environment_map: Option<Handle>,
    pub ambient_specular_brdf_integration_map: Option<Handle>,
    pub skybox_transform: Option<Mat4>,
    pub ambient_light: Option<Handle>,
    pub directional_light: Option<Handle>,
    pub directional_light_view_projections: Option<Vec<(f32, Mat4)>>,
    pub directional_light_view_projection_index: Option<usize>,
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
            projection_z_near: None,
            projection_z_far: None,
            projection_transform: Mat4::identity(),
            world_view_projection_transform: Default::default(),
            active_material: None,
            active_uv_test_texture_map: None,
            active_hdr_map: None,
            ambient_radiance_map: None,
            ambient_diffuse_irradiance_map: None,
            ambient_specular_prefiltered_environment_map: None,
            ambient_specular_brdf_integration_map: None,
            skybox_transform: None,
            ambient_light: None,
            directional_light: None,
            directional_light_view_projections: None,
            directional_light_view_projection_index: None,
            point_lights: vec![],
            spot_lights: vec![],
        }
    }
}

impl ShaderContext {
    pub fn get_world_transform(&mut self) -> Mat4 {
        self.world_transform
    }

    pub fn set_world_transform(&mut self, mat: Mat4) {
        self.world_transform = mat;

        self.recompute_world_view_transform();

        self.recompute_world_view_projection_transform();
    }

    pub fn set_view_position(&mut self, position: Vec4) {
        self.view_position = position;
    }

    pub fn set_view_inverse_transform(&mut self, mat: Mat4) {
        self.view_inverse_transform = mat;

        self.recompute_world_view_transform();

        self.recompute_world_view_projection_transform();
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection_transform
    }

    pub fn set_projection(&mut self, projection_transform: Mat4) {
        self.projection_transform = projection_transform;

        self.recompute_world_view_projection_transform();
    }

    pub fn to_ndc_space(&self, position_world_space: Vec3) -> Vec3 {
        let position_ndc_space = {
            let mut position_projection_space = Vec4::position(position_world_space)
                * self.view_inverse_transform
                * self.projection_transform;

            let w_inverse = 1.0 / position_projection_space.w;

            position_projection_space *= w_inverse;

            position_projection_space.x = (position_projection_space.x + 1.0) / 2.0;
            position_projection_space.y = (-position_projection_space.y + 1.0) / 2.0;

            position_projection_space
        };

        position_ndc_space.to_vec3()
    }

    pub fn set_ambient_light(&mut self, light: Option<Handle>) {
        self.ambient_light = light;
    }

    pub fn set_directional_light(&mut self, light: Option<Handle>) {
        self.directional_light = light;
    }

    pub fn set_directional_light_view_projections(&mut self, transforms: Option<Vec<(f32, Mat4)>>) {
        self.directional_light_view_projections = transforms;
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

    pub fn clear_lights(&mut self) {
        self.set_ambient_light(None);
        self.set_directional_light(None);
        self.get_point_lights_mut().clear();
        self.get_spot_lights_mut().clear();
    }

    pub fn set_active_material(&mut self, optional_handle: Option<Handle>) {
        self.active_material = optional_handle;
    }

    pub fn set_active_uv_test_texture_map(&mut self, optional_handle: Option<Handle>) {
        self.active_uv_test_texture_map = optional_handle;
    }

    pub fn set_ambient_radiance_map(&mut self, optional_handle: Option<Handle>) {
        self.ambient_radiance_map = optional_handle;
    }

    pub fn set_ambient_diffuse_irradiance_map(&mut self, optional_handle: Option<Handle>) {
        self.ambient_diffuse_irradiance_map = optional_handle;
    }

    pub fn set_ambient_specular_prefiltered_environment_map(
        &mut self,
        optional_handle: Option<Handle>,
    ) {
        self.ambient_specular_prefiltered_environment_map = optional_handle;
    }

    pub fn set_ambient_specular_brdf_integration_map(&mut self, optional_handle: Option<Handle>) {
        self.ambient_specular_brdf_integration_map = optional_handle;
    }

    pub fn set_active_hdr_map(&mut self, optional_handle: Option<Handle>) {
        self.active_hdr_map = optional_handle;
    }

    pub fn set_skybox_transform(&mut self, optional_transform: Option<Mat4>) {
        self.skybox_transform = optional_transform;
    }

    fn recompute_world_view_transform(&mut self) {
        self.world_view_transform = self.world_transform * self.view_inverse_transform;
    }

    fn recompute_world_view_projection_transform(&mut self) {
        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }
}
