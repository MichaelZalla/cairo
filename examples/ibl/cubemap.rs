use std::cell::RefCell;

use cairo::{
    buffer::framebuffer::Framebuffer,
    pipeline::{options::PipelineFaceCullingReject, Pipeline},
    resource::handle::Handle,
    scene::{camera::Camera, context::SceneContext},
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::{
        cubemap::{CubeMap, Side, CUBE_MAP_SIDES},
        map::{TextureBuffer, TextureMapStorageFormat},
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::shader::{
    HdrCubemapConvolutionFragmentShader, HdrEquirectangularProjectionFragmentShader,
    HdrEquirectangularProjectionVertexShader,
};

pub fn render_radiance_to_cubemap(
    hdr_texture_handle: &Handle,
    cubemap_size: u32,
    framebuffer_rc: &'static mut RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) -> CubeMap<Vec3> {
    {
        // Setup

        pipeline.set_vertex_shader(HdrEquirectangularProjectionVertexShader);

        pipeline.set_fragment_shader(HdrEquirectangularProjectionFragmentShader);

        pipeline.bind_framebuffer(Some(framebuffer_rc));

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_active_hdr_map(Some(*hdr_texture_handle));
    }

    let cubemap = render_scene_to_cubemap(
        cubemap_size,
        framebuffer_rc,
        scene_context,
        shader_context_rc,
        pipeline,
    );

    {
        // Cleanup

        pipeline.set_vertex_shader(DEFAULT_VERTEX_SHADER);

        pipeline.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        pipeline.bind_framebuffer(None);

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::Backfaces;

        shader_context_rc.borrow_mut().set_active_hdr_map(None);
    }

    cubemap
}

pub fn render_irradiance_to_cubemap(
    radiance_cubemap_texture_handle: &Handle,
    cubemap_size: u32,
    framebuffer_rc: &'static mut RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) -> CubeMap<Vec3> {
    {
        // Setup

        pipeline.set_fragment_shader(HdrCubemapConvolutionFragmentShader);

        pipeline.bind_framebuffer(Some(framebuffer_rc));

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_active_environment_map(Some(*radiance_cubemap_texture_handle));
    }

    let cubemap = render_scene_to_cubemap(
        cubemap_size,
        framebuffer_rc,
        scene_context,
        shader_context_rc,
        pipeline,
    );

    {
        // Cleanup

        pipeline.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        pipeline.bind_framebuffer(None);

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::Backfaces;

        shader_context_rc
            .borrow_mut()
            .set_active_environment_map(None);
    }

    cubemap
}

fn render_scene_to_cubemap(
    cubemap_size: u32,
    framebuffer_rc: &'static RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) -> CubeMap<Vec3> {
    let mut cubemap: CubeMap<Vec3> = Default::default();

    for side in &mut cubemap.sides {
        side.info.storage_format = TextureMapStorageFormat::Index8(0);
        side.width = cubemap_size;
        side.height = cubemap_size;
        side.is_loaded = true;
    }

    // Render each face of our cubemap.

    let mut cubemap_face_camera =
        Camera::from_perspective(Default::default(), vec3::FORWARD, 90.0, 1.0);

    for side in CUBE_MAP_SIDES {
        let face_direction = match side {
            Side::Front => vec3::FORWARD,
            Side::Back => vec3::FORWARD * -1.0,
            Side::Top => Vec3 {
                x: -0.0,
                y: 1.0,
                z: 0.0001,
            },
            Side::Bottom => Vec3 {
                x: -0.0,
                y: -1.0,
                z: 0.0001,
            },
            Side::Left => vec3::LEFT,
            Side::Right => vec3::LEFT * -1.0,
        };

        cubemap_face_camera
            .look_vector
            .set_target_position(face_direction);

        {
            let mut shader_context = shader_context_rc.borrow_mut();

            shader_context.set_view_position(Vec4::new(
                cubemap_face_camera.look_vector.get_position(),
                1.0,
            ));

            shader_context
                .set_view_inverse_transform(cubemap_face_camera.get_view_inverse_transform());

            shader_context.set_projection(cubemap_face_camera.get_projection());
        }

        let resources = (*scene_context.resources).borrow();
        let scene = &scene_context.scenes.borrow()[0];

        match scene.render(&resources, pipeline) {
            Ok(()) => {
                // Blit our framebuffer's color attachment buffer to our cubemap face texture.
                let framebuffer = framebuffer_rc.borrow();

                match &framebuffer.attachments.forward_or_deferred_hdr {
                    Some(hdr_attachment_rc) => {
                        let hdr_buffer = hdr_attachment_rc.borrow();

                        cubemap.sides[side as usize]
                            .levels
                            .push(TextureBuffer::<Vec3>(hdr_buffer.clone()));
                    }
                    None => (),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }

    cubemap
}
