use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::framebuffer::Framebuffer,
    color::Color,
    entity::Entity,
    material::cache::MaterialCache,
    matrix::Mat4,
    mesh::Mesh,
    resource::arena::Arena,
    scene::{
        camera::{frustum::Frustum, Camera},
        light::{PointLight, SpotLight},
    },
    shader::{fragment::FragmentShaderFn, geometry::GeometryShaderFn, vertex::VertexShaderFn},
};

pub trait Renderer {
    fn set_vertex_shader(&mut self, shader: VertexShaderFn);

    fn set_geometry_shader(&mut self, shader: GeometryShaderFn);

    fn set_fragment_shader(&mut self, shader: FragmentShaderFn);

    fn bind_framebuffer(&mut self, framebuffer_option: Option<Rc<RefCell<Framebuffer>>>);

    fn begin_frame(&mut self);

    fn end_frame(&mut self);

    fn render_frustum(&mut self, frustum: &Frustum, color: Option<Color>);

    fn render_camera(&mut self, camera: &Camera, color: Option<Color>);

    fn render_point_light(
        &mut self,
        light: &PointLight,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
    );

    fn render_spot_light(
        &mut self,
        light: &SpotLight,
        camera: Option<&Camera>,
        material_cache: Option<&mut MaterialCache>,
    );

    fn render_entity_aabb(
        &mut self,
        entity: &Entity,
        world_transform: &Mat4,
        mesh_arena: &Arena<Mesh>,
        color: Color,
    );

    fn render_entity(
        &mut self,
        world_transform: &Mat4,
        clipping_camera_frustum: &Option<Frustum>,
        entity_mesh: &Mesh,
        entity_material_name: &Option<String>,
    ) -> bool;
}
