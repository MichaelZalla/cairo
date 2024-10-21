use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::framebuffer::Framebuffer,
    render::culling::FaceCullingReject,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::GeometryShaderFn,
        vertex::VertexShaderFn,
    },
    software_renderer::SoftwareRenderer,
};

pub static SHADOW_MAP_CAMERA_NEAR: f32 = 0.05;
pub static DEFAULT_SHADOW_MAP_CAMERA_FAR: f32 = 1000.0;

#[derive(Debug, Clone)]
pub struct ShadowMapRenderingContext {
    pub projection_z_far: f32,
    pub framebuffer: Rc<RefCell<Framebuffer>>,
    pub shader_context: Rc<RefCell<ShaderContext>>,
    pub renderer: RefCell<SoftwareRenderer>,
}

impl ShadowMapRenderingContext {
    pub fn new(
        shadow_map_size: u32,
        projection_z_far: f32,
        vertex_shader: VertexShaderFn,
        geometry_shader: GeometryShaderFn,
        fragment_shader: FragmentShaderFn,
        scene_resources: Rc<SceneResources>,
    ) -> Self {
        // Shadow map framebuffer.

        let projection_z_near = SHADOW_MAP_CAMERA_NEAR;

        let framebuffer = {
            let mut framebuffer = Framebuffer::new(shadow_map_size, shadow_map_size);

            framebuffer.complete(projection_z_near, projection_z_far);

            Rc::new(RefCell::new(framebuffer))
        };

        // Shadow map shader context.

        let shader_context = Rc::new(RefCell::new(ShaderContext::default()));

        // Shadow map renderer.

        let renderer = {
            let mut renderer = SoftwareRenderer::new(
                shader_context.clone(),
                scene_resources,
                vertex_shader,
                fragment_shader,
                Default::default(),
            );

            renderer.set_geometry_shader(geometry_shader);

            renderer
                .options
                .rasterizer_options
                .face_culling_strategy
                .reject = FaceCullingReject::Frontfaces;

            renderer.bind_framebuffer(Some(framebuffer.clone()));

            RefCell::new(renderer)
        };

        // Shadow map rendering context.

        Self {
            projection_z_far,
            renderer,
            shader_context,
            framebuffer,
        }
    }
}
