use crate::{
    material::Material,
    matrix::Mat4,
    scene::light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
    vec::vec4::Vec4,
};

pub mod alpha;
pub mod fragment;
pub mod vertex;

#[derive(Default)]
pub struct ShaderContext {
    pub flags: u32,
    pub world_transform: Mat4,
    pub view_position: Vec4,
    pub view_inverse_transform: Mat4,
    pub world_view_transform: Mat4,
    pub projection_transform: Mat4,
    pub world_view_projection_transform: Mat4,
    pub default_specular_power: i32,
    pub active_material: Option<*const Material>,
    pub ambient_light: AmbientLight,
    pub directional_light: DirectionalLight,
    pub point_light: PointLight,
    pub spot_light: SpotLight,
}

impl ShaderContext {
    pub fn new(
        world_transform: Mat4,
        view_position: Vec4,
        view_inverse_transform: Mat4,
        projection_transform: Mat4,
        ambient_light: AmbientLight,
        directional_light: DirectionalLight,
        point_light: PointLight,
        spot_light: SpotLight,
    ) -> Self {
        Self {
            flags: 0,
            world_transform,
            view_position,
            view_inverse_transform,
            projection_transform,
            world_view_transform: world_transform * view_inverse_transform,
            world_view_projection_transform: world_transform
                * view_inverse_transform
                * projection_transform,
            default_specular_power: 8,
            active_material: None,
            ambient_light,
            directional_light,
            point_light,
            spot_light,
        }
    }

    pub fn get_projection(&self) -> Mat4 {
        return self.projection_transform;
    }

    pub fn set_projection(&mut self, projection_transform: Mat4) {
        self.projection_transform = projection_transform;
    }

    // @TODO Rename to set_view_position()
    pub fn set_camera_position(&mut self, position: Vec4) {
        self.view_position = position;
    }

    pub fn set_world_transform(&mut self, mat: Mat4) {
        self.world_transform = mat;

        self.world_view_transform = self.world_transform * self.view_inverse_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_view_inverse_transform(&mut self, mat: Mat4) {
        self.view_inverse_transform = mat;

        self.world_view_transform = self.world_transform * self.view_inverse_transform;

        self.world_view_projection_transform =
            self.world_view_transform * self.projection_transform;
    }

    pub fn set_ambient_light(&mut self, light: AmbientLight) {
        self.ambient_light = light;
    }

    pub fn set_directional_light(&mut self, light: DirectionalLight) {
        self.directional_light = light;
    }

    pub fn set_point_light(&mut self, light: PointLight) {
        self.point_light = light;
    }

    pub fn set_spot_light(&mut self, light: SpotLight) {
        self.spot_light = light;
    }

    pub fn set_active_material(&mut self, material_option: Option<*const Material>) {
        match material_option {
            Some(mat_raw_mut) => {
                self.active_material = Some(mat_raw_mut);
            }
            None => {
                self.active_material = None;
            }
        }
    }
}
