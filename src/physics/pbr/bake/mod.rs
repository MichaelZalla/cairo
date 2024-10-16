use std::{cell::RefCell, path::Path, rc::Rc};

use uuid::Uuid;

use crate::{
    buffer::framebuffer::Framebuffer,
    hdr::load::load_hdr,
    material::Material,
    physics::pbr::bake::shaders::{
        diffuse::HdrDiffuseIrradianceFragmentShader,
        equirectangular::{
            HdrEquirectangularProjectionFragmentShader, HdrEquirectangularProjectionVertexShader,
        },
        specular::HdrSpecularPrefilteredEnvironmentFragmentShader,
    },
    render::culling::FaceCullingReject,
    resource::handle::Handle,
    scene::{
        context::{utils::make_cube_scene, SceneContext},
        node::SceneNodeType,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::cubemap::CubeMap,
    vec::vec3::Vec3,
};

pub mod brdf;
pub mod shaders;

pub struct HDRBakeResult {
    pub radiance: CubeMap<Vec3>,
    pub diffuse_irradiance: CubeMap<Vec3>,
    pub specular_prefiltered_environment: CubeMap<Vec3>,
}

pub fn bake_diffuse_and_specular_from_hdri(hdr_filepath: &Path) -> Result<HDRBakeResult, String> {
    // Set up a simple cube scene, that we can use to render each side of a cubemap.

    let (scene_context, _shader_context) = make_cube_scene(1.0).unwrap();

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

    let hdr_texture_handle = (*scene_context.resources)
        .borrow_mut()
        .texture_vec3
        .borrow_mut()
        .insert(Uuid::new_v4(), hdr_texture);

    // Set up a renderer for cubemap rendering.

    let shader_context_rc: Rc<RefCell<ShaderContext>> = Default::default();

    let renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    let renderer_rc = RefCell::new(renderer);

    let cubemap_face_framebuffer = {
        let mut framebuffer = Framebuffer::new(0, 0);

        framebuffer.complete(0.3, 100.0);

        framebuffer
    };

    let cubemap_face_framebuffer_rc = Rc::new(RefCell::new(cubemap_face_framebuffer));

    // Generate a radiance cubemap texture from our HDR texture.

    let radiance = {
        {
            let mut framebuffer = cubemap_face_framebuffer_rc.borrow_mut();

            framebuffer.resize(1024, 1024, true);
        }

        render_radiance_to_cubemap(
            &hdr_texture_handle,
            cubemap_face_framebuffer_rc.clone(),
            &scene_context,
            &shader_context_rc,
            &renderer_rc,
        )
    };

    // Insert the radiance cubemap into the appropriate resource arena,
    // generating a handle.

    let radiance_cubemap_texture_handle: Handle;

    {
        radiance_cubemap_texture_handle = {
            (*scene_context.resources)
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
            cubemap_face_framebuffer_rc.clone(),
            &scene_context,
            &shader_context_rc,
            &renderer_rc,
        )
    };

    let specular_prefiltered_environment = {
        render_specular_prefiltered_environment_to_cubemap(
            &radiance_cubemap_texture_handle,
            cubemap_face_framebuffer_rc.clone(),
            &scene_context,
            &shader_context_rc,
            &renderer_rc,
        )
    };

    Ok(HDRBakeResult {
        radiance,
        diffuse_irradiance,
        specular_prefiltered_environment,
    })
}

fn render_radiance_to_cubemap(
    hdr_texture_handle: &Handle,
    framebuffer_rc: Rc<RefCell<Framebuffer>>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    renderer_rc: &RefCell<SoftwareRenderer>,
) -> CubeMap<Vec3> {
    {
        // Setup

        let mut renderer = renderer_rc.borrow_mut();

        renderer.set_vertex_shader(HdrEquirectangularProjectionVertexShader);

        renderer.set_fragment_shader(HdrEquirectangularProjectionFragmentShader);

        renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

        renderer.options.face_culling_strategy.reject = FaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_active_hdr_map(Some(*hdr_texture_handle));
    }

    let mut cubemap = CubeMap::<Vec3>::from_framebuffer(&framebuffer_rc.borrow());

    cubemap
        .render_scene(
            None,
            framebuffer_rc,
            scene_context,
            shader_context_rc,
            renderer_rc,
        )
        .unwrap();

    {
        // Cleanup

        let mut renderer = renderer_rc.borrow_mut();

        renderer.set_vertex_shader(DEFAULT_VERTEX_SHADER);

        renderer.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        renderer.bind_framebuffer(None);

        renderer.options.face_culling_strategy.reject = FaceCullingReject::Backfaces;

        shader_context_rc.borrow_mut().set_active_hdr_map(None);
    }

    cubemap
}

fn render_irradiance_to_cubemap(
    radiance_cubemap_texture_handle: &Handle,
    framebuffer_rc: Rc<RefCell<Framebuffer>>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    renderer_rc: &RefCell<SoftwareRenderer>,
) -> CubeMap<Vec3> {
    {
        // Setup

        let mut renderer = renderer_rc.borrow_mut();

        renderer.set_fragment_shader(HdrDiffuseIrradianceFragmentShader);

        renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

        renderer.options.face_culling_strategy.reject = FaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_ambient_radiance_map(Some(*radiance_cubemap_texture_handle));
    }

    let mut cubemap = CubeMap::<Vec3>::from_framebuffer(&framebuffer_rc.borrow());

    cubemap
        .render_scene(
            None,
            framebuffer_rc,
            scene_context,
            shader_context_rc,
            renderer_rc,
        )
        .unwrap();

    {
        // Cleanup

        let mut renderer = renderer_rc.borrow_mut();

        renderer.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        renderer.bind_framebuffer(None);

        renderer.options.face_culling_strategy.reject = FaceCullingReject::Backfaces;

        shader_context_rc
            .borrow_mut()
            .set_ambient_radiance_map(None);
    }

    cubemap
}

fn render_specular_prefiltered_environment_to_cubemap(
    radiance_cubemap_texture_handle: &Handle,
    framebuffer_rc: Rc<RefCell<Framebuffer>>,
    scene_context: &SceneContext,
    shader_context_rc: &RefCell<ShaderContext>,
    renderer_rc: &RefCell<SoftwareRenderer>,
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

        let mut renderer = renderer_rc.borrow_mut();

        renderer.set_fragment_shader(HdrSpecularPrefilteredEnvironmentFragmentShader);

        renderer.options.face_culling_strategy.reject = FaceCullingReject::None;

        shader_context_rc
            .borrow_mut()
            .set_ambient_radiance_map(Some(*radiance_cubemap_texture_handle));
    }

    let mut cubemap = CubeMap::<Vec3>::from_framebuffer(&framebuffer_rc.borrow());

    for side in cubemap.sides.as_mut() {
        side.generate_mipmaps().unwrap();
    }

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

            let mut renderer = renderer_rc.borrow_mut();

            renderer.bind_framebuffer(Some(framebuffer_rc.clone()));
        }

        println!(
            "{}: {}x{}",
            mipmap_level, mipmap_dimension, mipmap_dimension
        );

        cubemap
            .render_scene(
                Some(mipmap_level),
                framebuffer_rc.clone(),
                scene_context,
                shader_context_rc,
                renderer_rc,
            )
            .unwrap();
    }

    {
        // Cleanup

        let mut renderer = renderer_rc.borrow_mut();

        renderer.set_fragment_shader(DEFAULT_FRAGMENT_SHADER);

        renderer.bind_framebuffer(None);

        renderer.options.face_culling_strategy.reject = FaceCullingReject::Backfaces;

        shader_context_rc
            .borrow_mut()
            .set_ambient_radiance_map(None);
    }

    cubemap
}
