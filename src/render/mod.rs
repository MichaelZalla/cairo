use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::framebuffer::Framebuffer,
    shader::{fragment::FragmentShaderFn, geometry::GeometryShaderFn, vertex::VertexShaderFn},
};

pub trait Renderer {
    fn set_vertex_shader(&mut self, shader: VertexShaderFn);
    fn set_geometry_shader(&mut self, shader: GeometryShaderFn);
    fn set_fragment_shader(&mut self, shader: FragmentShaderFn);
    fn bind_framebuffer(&mut self, framebuffer_option: Option<Rc<RefCell<Framebuffer>>>);
}
