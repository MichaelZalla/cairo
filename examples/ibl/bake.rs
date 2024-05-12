use std::{cell::RefCell, path::Path};

use cairo::{
    buffer::framebuffer::Framebuffer,
    hdr::load::load_hdr,
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

use uuid::Uuid;

use crate::{
    scene,
    shaders::{
        diffuse::HdrDiffuseIrradianceFragmentShader,
        equirectangular::{
            HdrEquirectangularProjectionFragmentShader, HdrEquirectangularProjectionVertexShader,
        },
    },
};

pub fn bake_diffuse_irradiance_for_hdri(
    hdr_filepath: &Path,
) -> Result<(CubeMap<Vec3>, CubeMap<Vec3>), String> {
    // Set up a simple cube scene, that we can use to render each side of a cubemap.

    let cube_scene_context = scene::make_cube_scene(1.0).unwrap();

    // Load the HDR image data into a texture.

    let hdr_texture = match load_hdr(hdr_filepath) {
        Ok(hdr) => {
            println!("{:?}", hdr.source);
            println!("{:?}", hdr.headers);
            println!("Decoded {} bytes from file.", hdr.bytes.len());

            hdr.to_texture_map()
        }
        Err(e) => {
            panic!("{}", format!("Failed to read HDR file: {}", e).to_string());
        }
    };

    println!("{}x{}", hdr_texture.width, hdr_texture.height);

    // Store the texture in our scene resources' HDR texture arena.

    let hdr_texture_handle = (*cube_scene_context.resources)
        .borrow_mut()
        .texture_vec3
        .borrow_mut()
        .insert(Uuid::new_v4(), hdr_texture);

    // Set up a pipeline for rendering our cubemaps.

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        cube_scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    // Generate a radiance cubemap texture from our HDR texture.

    let radiance_cubemap = {
        static CUBEMAP_SIZE: u32 = 1024;

        let cubemap_face_framebuffer = {
            let mut framebuffer = Framebuffer::new(CUBEMAP_SIZE, CUBEMAP_SIZE);

            framebuffer.complete(0.3, 100.0);

            framebuffer
        };

        let cubemap_face_framebuffer_rc =
            Box::leak(Box::new(RefCell::new(cubemap_face_framebuffer)));

        render_radiance_to_cubemap(
            &hdr_texture_handle,
            cubemap_face_framebuffer_rc,
            &cube_scene_context,
            &shader_context_rc,
            &mut pipeline,
        )
    };

    // Generate an (approximate) irradiance cubemap texture from our radiance
    // cubemap texture.

    let irradiance_cubemap = {
        static CUBEMAP_SIZE: u32 = 32;

        let cubemap_face_framebuffer = {
            let mut framebuffer = Framebuffer::new(CUBEMAP_SIZE, CUBEMAP_SIZE);

            framebuffer.complete(0.3, 100.0);

            framebuffer
        };

        let cubemap_face_framebuffer_rc =
            Box::leak(Box::new(RefCell::new(cubemap_face_framebuffer)));

        let radiance_cubemap_texture_handle = {
            (*cube_scene_context.resources)
                .borrow_mut()
                .cubemap_vec3
                .borrow_mut()
                .insert(Uuid::new_v4(), radiance_cubemap.clone())
        };

        render_irradiance_to_cubemap(
            &radiance_cubemap_texture_handle,
            cubemap_face_framebuffer_rc,
            &cube_scene_context,
            &shader_context_rc,
            &mut pipeline,
        )
    };

    Ok((radiance_cubemap, irradiance_cubemap))
}

fn render_radiance_to_cubemap(
    hdr_texture_handle: &Handle,
    framebuffer_rc: &'static RefCell<Framebuffer>,
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

    let cubemap =
        render_scene_to_cubemap(framebuffer_rc, scene_context, shader_context_rc, pipeline);

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

fn render_irradiance_to_cubemap(
    radiance_cubemap_texture_handle: &Handle,
    framebuffer_rc: &'static mut RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) -> CubeMap<Vec3> {
    {
        // Setup

        pipeline.set_fragment_shader(HdrDiffuseIrradianceFragmentShader);

        pipeline.bind_framebuffer(Some(framebuffer_rc));

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_active_ambient_diffuse_irradiance_map(Some(*radiance_cubemap_texture_handle));
    }

    let cubemap =
        render_scene_to_cubemap(framebuffer_rc, scene_context, shader_context_rc, pipeline);

    {
        // Cleanup

        pipeline.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        pipeline.bind_framebuffer(None);

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::Backfaces;

        shader_context_rc
            .borrow_mut()
            .set_active_ambient_diffuse_irradiance_map(None);
    }

    cubemap
}

fn render_scene_to_cubemap(
    framebuffer_rc: &'static RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) -> CubeMap<Vec3> {
    let cubemap_size = {
        let framebuffer = framebuffer_rc.borrow();

        assert_eq!(framebuffer.width, framebuffer.height);

        framebuffer.width
    };

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
