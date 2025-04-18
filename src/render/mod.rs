use options::RenderOptions;

use crate::{
    color::Color,
    geometry::primitives::{aabb::AABB, ray::Ray},
    matrix::Mat4,
    mesh::Mesh,
    resource::handle::Handle,
    scene::{
        camera::{frustum::Frustum, Camera},
        empty::EmptyDisplayKind,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
    },
    texture::cubemap::CubeMap,
    vec::vec3::Vec3,
};

pub mod culling;
pub mod options;
pub mod viewport;

pub trait Renderer {
    fn get_options(&self) -> &RenderOptions;

    fn get_options_mut(&mut self) -> &mut RenderOptions;

    fn on_camera_update(&self, camera: &Camera);

    fn begin_frame(&mut self);

    fn end_frame(&mut self);

    fn render_text(&mut self, transform: &Mat4, color: Color, text: &str) -> Result<(), String>;

    fn render_point(
        &mut self,
        transform: &Mat4,
        color: Option<Color>,
        mesh: Option<&Mesh>,
        material_handle: Option<Handle>,
    );

    fn render_line(&mut self, start_world_space: Vec3, end_world_space: Vec3, color: Color);

    fn render_circle(&mut self, position: &Vec3, radius_world_units: f32, color: Color);

    fn render_axes(&mut self, transform: Option<&Mat4>);

    fn render_ground_plane(&mut self, parallels: usize);

    fn render_empty(
        &mut self,
        transform: &Mat4,
        display_kind: EmptyDisplayKind,
        with_basis_vectors: bool,
        color: Option<Color>,
    );

    fn render_frustum(&mut self, frustum: &Frustum, color: Option<Color>);

    fn render_camera(&mut self, camera: &Camera, color: Option<Color>);

    fn render_ambient_light(&mut self, transform: &Mat4, light: &AmbientLight);

    fn render_directional_light(&mut self, transform: &Mat4, light: &DirectionalLight);

    fn render_point_light(&mut self, transform: &Mat4, light: &PointLight);

    fn render_spot_light(&mut self, transform: &Mat4, light: &SpotLight);

    fn render_ray(&mut self, ray: &Ray, color: Color);

    fn render_aabb(&mut self, aabb: &AABB, world_transform: Option<&Mat4>, color: Color);

    fn render_entity(
        &mut self,
        world_transform: &Mat4,
        entity_mesh: &Mesh,
        entity_material: &Option<Handle>,
    ) -> bool;

    // @TODO Skybox holds a Transform.
    fn render_skybox(&mut self, skybox: &CubeMap, camera: &Camera, skybox_rotation: Option<Mat4>);

    // @TODO Skybox holds a Transform.
    fn render_skybox_hdr(
        &mut self,
        skybox_hdr: &CubeMap<Vec3>,
        camera: &Camera,
        skybox_rotation: Option<Mat4>,
    );
}
