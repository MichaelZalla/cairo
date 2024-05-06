use std::{borrow::BorrowMut, cell::RefCell, path::Path};

use shader::AmbientDiffuseCubemapFragmentShader;
use uuid::Uuid;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    hdr::load::load_hdr,
    matrix::Mat4,
    pipeline::Pipeline,
    resource::handle::Handle,
    scene::node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod},
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
};

use crate::cubemap::{render_irradiance_to_cubemap, render_radiance_to_cubemap};

pub mod cubemap;
pub mod scene;
pub mod shader;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ibl".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Scene context

    let mut scene_context =
        scene::make_cube_scene(framebuffer_rc.borrow().width_over_height).unwrap();

    // Load an HDR image as an available texture.

    let hdr_texture_handle: Option<Handle>;

    // let hdr_filepath = Path::new("./examples/ibl/assets/rural_asphalt_road_4k.hdr");
    let hdr_filepath = Path::new("./examples/ibl/assets/poly_haven_studio_4k.hdr");

    match load_hdr(hdr_filepath) {
        Ok(hdr) => {
            println!("{:?}", hdr.source);
            println!("{:?}", hdr.headers);
            println!("Decoded {} bytes from file.", hdr.bytes.len());

            let hdr_texture = hdr.to_texture_map();

            println!("{}x{}", hdr_texture.width, hdr_texture.height);

            hdr_texture_handle = Some(
                (*scene_context.resources)
                    .borrow_mut()
                    .hdr
                    .borrow_mut()
                    .insert(Uuid::new_v4(), hdr_texture),
            );
        }
        Err(e) => {
            panic!("{}", format!("Failed to read HDR file: {}", e).to_string());
        }
    }

    // Shader context

    let shader_context = ShaderContext::default();

    let shader_context_rc = RefCell::new(shader_context);

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    // Generate a cubemap texture from our (projected) radiance texture.

    let preconvolution_ambient_diffuse_handle = {
        static CUBEMAP_SIZE: u32 = 1024;

        let cubemap_face_framebuffer = {
            let mut framebuffer = Framebuffer::new(CUBEMAP_SIZE, CUBEMAP_SIZE);

            framebuffer.complete(0.3, 100.0);

            framebuffer
        };

        let cubemap_face_framebuffer_rc =
            Box::leak(Box::new(RefCell::new(cubemap_face_framebuffer)));

        let scene_context = scene_context.borrow_mut();

        let cubemap_hdr = render_radiance_to_cubemap(
            &hdr_texture_handle.unwrap(),
            CUBEMAP_SIZE,
            cubemap_face_framebuffer_rc,
            scene_context,
            &shader_context_rc,
            &mut pipeline,
        );

        (*scene_context.resources)
            .borrow_mut()
            .skybox_hdr
            .borrow_mut()
            .insert(Uuid::new_v4(), cubemap_hdr)
    };

    // Convolute the resulting cubemap texture, approximating irradiance.

    let postconvolution_ambient_diffuse_handle = {
        static CUBEMAP_SIZE: u32 = 32;

        let cubemap_face_framebuffer = {
            let mut framebuffer = Framebuffer::new(CUBEMAP_SIZE, CUBEMAP_SIZE);

            framebuffer.complete(0.3, 100.0);

            framebuffer
        };

        let cubemap_face_framebuffer_rc =
            Box::leak(Box::new(RefCell::new(cubemap_face_framebuffer)));

        let scene_context = scene_context.borrow_mut();

        let cubemap_hdr = render_irradiance_to_cubemap(
            &preconvolution_ambient_diffuse_handle,
            CUBEMAP_SIZE,
            cubemap_face_framebuffer_rc,
            scene_context,
            &shader_context_rc,
            &mut pipeline,
        );

        (*scene_context.resources)
            .borrow_mut()
            .skybox_hdr
            .borrow_mut()
            .insert(Uuid::new_v4(), cubemap_hdr)
    };

    // Set the convoluted ambient diffuse map as our current "environment" map.

    shader_context_rc
        .borrow_mut()
        .set_active_environment_map(Some(postconvolution_ambient_diffuse_handle));

    let scene_context_rc = RefCell::new(scene_context);

    pipeline.set_fragment_shader(AmbientDiffuseCubemapFragmentShader);

    let pipeline_rc = RefCell::new(pipeline);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let scene_context = scene_context_rc.borrow_mut();
        let resources = (*scene_context.resources).borrow();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = shader_context_rc.borrow_mut();

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        // Traverse the scene graph and update its nodes.

        let mut update_scene_graph_node = |_current_depth: usize,
                                           _current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            node.update(
                &resources,
                app,
                mouse_state,
                keyboard_state,
                game_controller_state,
                &mut shader_context,
            )
        };

        scenes[0].root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut update_scene_graph_node,
        )?;

        let mut pipeline = pipeline_rc.borrow_mut();

        pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        pipeline
            .geometry_shader_options
            .update(keyboard_state, mouse_state, game_controller_state);

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our scene.

        // Render scene.

        let scene_context = scene_context_rc.borrow();
        let resources = (*scene_context.resources).borrow();

        let mut pipeline = pipeline_rc.borrow_mut();

        pipeline.bind_framebuffer(Some(&framebuffer_rc));

        let scene = &scene_context.scenes.borrow()[0];

        match scene.render(&resources, &mut pipeline) {
            Ok(()) => {
                // Write out.

                let framebuffer = framebuffer_rc.borrow();

                match framebuffer.attachments.color.as_ref() {
                    Some(color_buffer_lock) => {
                        let color_buffer = color_buffer_lock.borrow();

                        Ok(color_buffer.get_all().clone())
                    }
                    None => panic!(),
                }
            }
            Err(e) => panic!("{}", e),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
