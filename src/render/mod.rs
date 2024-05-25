use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::framebuffer::Framebuffer,
    matrix::Mat4,
    mesh::Mesh,
    scene::camera::frustum::Frustum,
    shader::{fragment::FragmentShaderFn, geometry::GeometryShaderFn, vertex::VertexShaderFn},
};

pub trait Renderer {
    fn set_vertex_shader(&mut self, shader: VertexShaderFn);
    fn set_geometry_shader(&mut self, shader: GeometryShaderFn);
    fn set_fragment_shader(&mut self, shader: FragmentShaderFn);
    fn bind_framebuffer(&mut self, framebuffer_option: Option<Rc<RefCell<Framebuffer>>>);
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn render_entity(
        &mut self,
        world_transform: &Mat4,
        clipping_camera_frustum: &Option<Frustum>,
        entity_mesh: &Mesh,
        entity_material_name: &Option<String>,
    ) -> bool;
}
