extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    color::Color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::{
        options::{
            tone_mapping::ToneMappingOperator, RenderOptions, RenderPassFlags,
        },
        Renderer,
    },
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    software_renderer::SoftwareRenderer,
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::vec3::Vec3,
};

use scene::make_scene;

mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/bloom".to_string(),
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        make_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut point_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            &mut texture_u8_arena,
            rendering_context,
        )
    }?;

    {
        let scenes = &mut scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer =
        SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

    renderer.options = {
        let flags: RenderPassFlags = Default::default();

        let bloom_dirt_mask_handle = {
            let mut texture_u8_arena = scene_context.resources.texture_u8.borrow_mut();

            let mut map = TextureMap::new(
                "./examples/bloom/assets/dirt_mask.png",
                TextureMapStorageFormat::RGB24,
            );

            map.load(rendering_context)?;

            texture_u8_arena.insert(map)
        };

        RenderOptions {
            render_pass_flags: flags | RenderPassFlags::BLOOM,
            bloom_dirt_mask_handle: Some(bloom_dirt_mask_handle),
            ..Default::default()
        }
    };

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.emissive_color_mapping_active = true;

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let update_node = |_current_world_transform: &Mat4,
                       node: &mut SceneNode,
                       resources: &SceneResources,
                       _app: &App,
                       _mouse_state: &MouseState,
                       _keyboard_state: &KeyboardState,
                       _game_controller_state: &GameControllerState,
                       _shader_context: &mut ShaderContext|
     -> Result<bool, String> {
        let (node_type, handle) = (node.get_type(), node.get_handle());

        match node_type {
            SceneNodeType::Entity => match handle {
                Some(handle) => {
                    let mesh_arena = resources.mesh.borrow();
                    let mut entity_arena = resources.entity.borrow_mut();

                    match entity_arena.get_mut(handle) {
                        Ok(entry) => {
                            let entity = &mut entry.item;

                            if let Ok(entry) = mesh_arena.get(&entity.mesh) {
                                let mesh = &entry.item;

                                if let Some(object_name) = &mesh.object_name
                                    && object_name == "plane"
                                {
                                    return Ok(false);
                                }
                            }

                            Ok(false)
                        }
                        Err(err) => panic!(
                            "Failed to get Entity from Arena with Handle {:?}: {}",
                            handle, err
                        ),
                    }
                }
                None => {
                    panic!("Encountered a `Entity` node with no resource handle!")
                }
            },
            _ => Ok(false),
        }
    };

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let mut shader_context = (*shader_context_rc).borrow_mut();

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        let update_node_rc = Rc::new(update_node);

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_node_rc),
        )?;

        let mut renderer = renderer_rc.borrow_mut();

        renderer.update(keyboard_state);

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();
        }

        // Render scene.

        scene.render(resources, &renderer_rc, None)?;

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.end_frame();
        }

        // Write out.

        let framebuffer = framebuffer_rc.borrow();

        match (
            framebuffer.attachments.color.as_ref(),
            framebuffer.attachments.bloom.as_ref(),
        ) {
            (Some(color_buffer_rc), Some(bloom_texture_map_rc)) => {
                let color_buffer = color_buffer_rc.borrow();

                color_buffer.copy_to(canvas);

                if false {
                    let bloom_texture_map = bloom_texture_map_rc.borrow();

                    blit_bloom_mipmaps_to_canvas(&bloom_texture_map, canvas);
                }

                Ok(())
            }
            _ => panic!(),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn blit_bloom_mipmaps_to_canvas(bloom_texture_map: &TextureMap<Vec3>, canvas: &mut [u8]) {
    let width = bloom_texture_map.levels[0].0.width;

    let (mut thumbnail_start_x, thumbnail_start_y) = (0, 0);

    for level_index in 1..bloom_texture_map.levels.len() {
        let mipmap = &bloom_texture_map.levels[level_index].0;

        for x in 0..mipmap.width {
            for y in 0..mipmap.height {
                let canvas_index =
                    (((thumbnail_start_y + y) * width + (thumbnail_start_x + x)) * 4) as usize;

                let bloom_color_hdr = mipmap.get(x, y);

                static TONE_MAPPING_OPERATOR: ToneMappingOperator =
                    ToneMappingOperator::Exposure(1.0);

                let mut bloom_color_tone_mapped_linear =
                    TONE_MAPPING_OPERATOR.map(*bloom_color_hdr);

                bloom_color_tone_mapped_linear.linear_to_srgb();

                let bloom_color_ldr = Color::from_vec3(bloom_color_tone_mapped_linear * 255.0);

                canvas[canvas_index] = bloom_color_ldr.r as u8;
                canvas[canvas_index + 1] = bloom_color_ldr.g as u8;
                canvas[canvas_index + 2] = bloom_color_ldr.b as u8;
            }
        }

        thumbnail_start_x += mipmap.width;
    }
}
