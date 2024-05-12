use std::{cell::RefCell, path::Path};

use uuid::Uuid;

use cairo::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    hdr::load::load_hdr,
    material::Material,
    physics::pbr::{geometry_smith_indirect, importance_sample_ggx},
    pipeline::{options::PipelineFaceCullingReject, Pipeline},
    random::hammersley_2d_sequence,
    resource::handle::Handle,
    scene::{camera::Camera, context::SceneContext, node::SceneNodeType},
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::{
        cubemap::{CubeMap, CUBE_MAP_SIDES},
        map::{TextureBuffer, TextureMap},
    },
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use crate::{
    scene,
    shaders::{
        diffuse::HdrDiffuseIrradianceFragmentShader,
        equirectangular::{
            HdrEquirectangularProjectionFragmentShader, HdrEquirectangularProjectionVertexShader,
        },
        specular::HdrSpecularPrefilteredEnvironmentFragmentShader,
    },
};

pub struct HDRBakeResult {
    pub radiance: CubeMap<Vec3>,
    pub diffuse_irradiance: CubeMap<Vec3>,
    pub specular_prefiltered_environment: CubeMap<Vec3>,
    pub specular_brdf_integration: TextureMap<Vec2>,
}

pub fn bake_diffuse_and_specular_from_hdri(hdr_filepath: &Path) -> Result<HDRBakeResult, String> {
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

    let cubemap_face_framebuffer = {
        let mut framebuffer = Framebuffer::new(0, 0);

        framebuffer.complete(0.3, 100.0);

        framebuffer
    };

    let cubemap_face_framebuffer_rc = Box::leak(Box::new(RefCell::new(cubemap_face_framebuffer)));

    // Generate a radiance cubemap texture from our HDR texture.

    let radiance = {
        {
            let mut framebuffer = cubemap_face_framebuffer_rc.borrow_mut();

            framebuffer.resize(1024, 1024, true);
        }

        render_radiance_to_cubemap(
            &hdr_texture_handle,
            cubemap_face_framebuffer_rc,
            &cube_scene_context,
            &shader_context_rc,
            &mut pipeline,
        )
    };

    // Insert the radiance cubemap into the appropriate resource arena,
    // generating a handle.

    let radiance_cubemap_texture_handle: Handle;

    {
        radiance_cubemap_texture_handle = {
            (*cube_scene_context.resources)
                .borrow_mut()
                .cubemap_vec3
                .borrow_mut()
                .insert(Uuid::new_v4(), radiance.clone())
        };
    }

    // Generate an (approximate) irradiance cubemap texture from our radiance
    // cubemap texture.

    let diffuse_irradiance = {
        {
            let mut framebuffer = cubemap_face_framebuffer_rc.borrow_mut();

            framebuffer.resize(32, 32, true);
        }

        render_irradiance_to_cubemap(
            &radiance_cubemap_texture_handle,
            cubemap_face_framebuffer_rc,
            &cube_scene_context,
            &shader_context_rc,
            &mut pipeline,
        )
    };

    let specular_prefiltered_environment = {
        render_specular_prefiltered_environment_to_cubemap(
            &radiance_cubemap_texture_handle,
            cubemap_face_framebuffer_rc,
            &cube_scene_context,
            &shader_context_rc,
            &mut pipeline,
        )
    };

    let specular_brdf_integration = generate_specular_brdf_integration_map(512);

    Ok(HDRBakeResult {
        radiance,
        diffuse_irradiance,
        specular_prefiltered_environment,
        specular_brdf_integration,
    })
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

    let mut cubemap = make_cubemap(framebuffer_rc, false).unwrap();

    render_scene_to_cubemap(
        &mut cubemap,
        None,
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

fn render_irradiance_to_cubemap(
    radiance_cubemap_texture_handle: &Handle,
    framebuffer_rc: &'static RefCell<Framebuffer>,
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
            .set_active_ambient_radiance_map(Some(*radiance_cubemap_texture_handle));
    }

    let mut cubemap = make_cubemap(framebuffer_rc, false).unwrap();

    render_scene_to_cubemap(
        &mut cubemap,
        None,
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
            .set_active_ambient_radiance_map(None);
    }

    cubemap
}

fn render_specular_prefiltered_environment_to_cubemap(
    radiance_cubemap_texture_handle: &Handle,
    framebuffer_rc: &'static RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) -> CubeMap<Vec3> {
    let material_name = "specular_roughness".to_string();

    {
        // Setup

        let material = Material {
            name: material_name.clone(),
            roughness: 0.0,
            ..Default::default()
        };

        let resources = (*scene_context.resources).borrow_mut();

        resources.material.borrow_mut().insert(material);

        //

        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        let cube_entity_handle = scene
            .root
            .find(&mut |node| *node.get_type() == SceneNodeType::Entity)
            .unwrap()
            .unwrap();

        if let Ok(entry) = resources.entity.borrow_mut().get_mut(&cube_entity_handle) {
            let entity = &mut entry.item;

            entity.material = Some(material_name.clone());
        }

        //

        // framebuffer_rc.borrow_mut().resize(32, 32, true);
        framebuffer_rc.borrow_mut().resize(128, 128, true);

        pipeline.set_fragment_shader(HdrSpecularPrefilteredEnvironmentFragmentShader);

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_active_ambient_radiance_map(Some(*radiance_cubemap_texture_handle));
    }

    let mut cubemap = make_cubemap(framebuffer_rc, true).unwrap();

    let lods = cubemap.sides[0].levels.len();

    for mipmap_level in 0..lods.min(5) {
        let mipmap_level_alpha = (mipmap_level as f32) / (5.0 - 1.0);

        let mipmap_dimension = cubemap.sides[0].levels[mipmap_level].0.width;

        {
            let resources = (*scene_context.resources).borrow_mut();

            let mut materials = resources.material.borrow_mut();

            let material = materials.get_mut(&material_name).unwrap();

            material.roughness = mipmap_level_alpha;

            framebuffer_rc
                .borrow_mut()
                .resize(mipmap_dimension, mipmap_dimension, true);

            pipeline.bind_framebuffer(Some(framebuffer_rc));
        }

        println!(
            "{}: {}x{}",
            mipmap_level, mipmap_dimension, mipmap_dimension
        );

        render_scene_to_cubemap(
            &mut cubemap,
            Some(mipmap_level),
            framebuffer_rc,
            scene_context,
            shader_context_rc,
            pipeline,
        );
    }

    {
        // Cleanup

        pipeline.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        pipeline.bind_framebuffer(None);

        pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::Backfaces;

        shader_context_rc
            .borrow_mut()
            .set_active_ambient_radiance_map(None);
    }

    cubemap
}

fn render_scene_to_cubemap(
    cubemap: &mut CubeMap<Vec3>,
    mipmap_level: Option<usize>,
    framebuffer_rc: &'static RefCell<Framebuffer>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    pipeline: &mut Pipeline,
) {
    // Render each face of our cubemap.

    let mut cubemap_face_camera =
        Camera::from_perspective(Default::default(), vec3::FORWARD, 90.0, 1.0);

    for side in CUBE_MAP_SIDES {
        cubemap_face_camera
            .look_vector
            .set_target_position(side.get_direction());

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

                        cubemap.sides[side as usize].levels[mipmap_level.unwrap_or(0)] =
                            TextureBuffer::<Vec3>(hdr_buffer.clone());
                    }
                    None => (),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}

fn make_cubemap(
    framebuffer_rc: &'static RefCell<Framebuffer>,
    generate_mipmaps: bool,
) -> Result<CubeMap<Vec3>, String> {
    let cubemap_size = {
        let framebuffer = framebuffer_rc.borrow();

        assert_eq!(framebuffer.width, framebuffer.height);

        framebuffer.width
    };

    let mut texture_map = TextureMap::from_buffer(
        cubemap_size,
        cubemap_size,
        Buffer2D::<Vec3>::new(cubemap_size, cubemap_size, None),
    );

    let mut cubemap: CubeMap<Vec3> = Default::default();

    if generate_mipmaps {
        texture_map.generate_mipmaps()?;
    }

    for side_index in 0..6 {
        cubemap.sides[side_index] = texture_map.clone();
    }

    Ok(cubemap)
}

fn generate_specular_brdf_integration_map(size: u32) -> TextureMap<Vec2> {
    let mut map = TextureMap::from_buffer(size, size, Buffer2D::<Vec2>::new(512, 512, None));

    // Integrate specular BRDF over angle and roughness (axes).

    let one_over_size_doubled = 1.0 / (size as f32) / 2.0;

    for y in 0..size {
        let y_alpha = one_over_size_doubled / 2.0 + y as f32 / (size + 1) as f32;

        for x in 0..size {
            let x_alpha = one_over_size_doubled / 2.0 + x as f32 / (size + 1) as f32;

            let likeness_to_view_direction = x_alpha;
            let roughness = y_alpha;

            map.levels[0].0.set(
                x,
                size - 1 - y,
                integrate_brdf(likeness_to_view_direction, roughness),
            );
        }
    }

    map
}

fn integrate_brdf(normal_likeness_to_view_direction: f32, roughness: f32) -> Vec2 {
    let direction_to_view_position = Vec3 {
        x: (1.0 - normal_likeness_to_view_direction * normal_likeness_to_view_direction).sqrt(),
        y: 0.0,
        z: normal_likeness_to_view_direction,
    };

    let mut accumulated_scale: f32 = 0.0;
    let mut accumulated_bias: f32 = 0.0;

    let normal = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.999,
    };

    static SAMPLE_COUNT: usize = 1024;

    let one_over_n = 1.0 / SAMPLE_COUNT as f32;

    for i in 0..SAMPLE_COUNT {
        let random_direction_hammersley = hammersley_2d_sequence(i as u32, one_over_n);

        let biased_sample_direction =
            importance_sample_ggx(random_direction_hammersley, &normal, roughness);

        let direction_to_environment_light = (biased_sample_direction
            * (2.0 * direction_to_view_position.dot(biased_sample_direction))
            - direction_to_view_position)
            .as_normal();

        let normal_likeness_to_environment_light = (direction_to_environment_light.z).max(0.0);

        let normal_likeness_to_biased_sample_direction = (biased_sample_direction.z).max(0.0);

        let view_likeness_to_biased_sample_direction = direction_to_view_position
            .dot(biased_sample_direction)
            .max(0.0);

        if normal_likeness_to_environment_light > 0.0
            && normal_likeness_to_biased_sample_direction > 0.0
        {
            let g = geometry_smith_indirect(
                &normal,
                &direction_to_view_position,
                &direction_to_environment_light,
                roughness,
            );

            debug_assert!(
                normal_likeness_to_biased_sample_direction != 0.0
                    && normal_likeness_to_view_direction != 0.0,
                "{}, {}, {}",
                normal_likeness_to_view_direction,
                roughness,
                normal_likeness_to_biased_sample_direction
            );

            let g_vis = (g * view_likeness_to_biased_sample_direction)
                / (normal_likeness_to_biased_sample_direction * normal_likeness_to_view_direction);

            let fc = (1.0 - view_likeness_to_biased_sample_direction).powi(5);

            debug_assert!(
                !fc.is_nan(),
                "{:?}, {:?}",
                normal_likeness_to_view_direction,
                roughness
            );

            accumulated_scale += (1.0 - fc) * g_vis;
            accumulated_bias += fc * g_vis;
        }
    }

    let scale = accumulated_scale / SAMPLE_COUNT as f32;
    let bias = accumulated_bias / SAMPLE_COUNT as f32;

    Vec2 {
        x: scale,
        y: bias,
        z: 0.0,
    }
}
