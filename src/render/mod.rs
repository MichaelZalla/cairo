use crate::{
    color::Color,
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::Mesh,
    resource::{arena::Arena, handle::Handle},
    scene::{
        camera::{frustum::Frustum, Camera},
        light::{point_light::PointLight, spot_light::SpotLight},
    },
    texture::cubemap::CubeMap,
    vec::vec3::Vec3,
};

pub mod culling;
pub mod options;
pub mod viewport;

pub trait Renderer {
    fn begin_frame(&mut self);

    fn end_frame(&mut self);

    fn render_point(
        &mut self,
        point_world_space: Vec3,
        color: &Color,
        camera: Option<&Camera>,
        materials: Option<&mut Arena<Material>>,
        material: Option<Handle>,
        scale: Option<f32>,
    );

    fn render_line(&mut self, start_world_space: Vec3, end_world_space: Vec3, color: &Color);

    fn render_point_indicator(&mut self, position: Vec3, scale: f32);

    fn render_world_axes(&mut self, scale: f32);

    fn render_ground_plane(&mut self, scale: f32);

    fn render_frustum(&mut self, frustum: &Frustum, color: Option<&Color>);

    fn render_camera(&mut self, camera: &Camera, color: Option<&Color>);

    fn render_point_light(&mut self, light: &PointLight);

    fn render_spot_light(&mut self, light: &SpotLight);

    fn render_entity_aabb(
        &mut self,
        entity: &Entity,
        world_transform: &Mat4,
        meshes: &Arena<Mesh>,
        wireframe_color: &Vec3,
    );

    fn render_entity(
        &mut self,
        world_transform: &Mat4,
        clipping_camera_frustum: &Option<Frustum>,
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
