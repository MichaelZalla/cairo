use std::{cell::RefCell, f32::consts::PI, path::Path, rc::Rc};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    scene::{
        context::SceneContext,
        graph::SceneGraph,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3,
};

use scene::make_sphere_grid_scene;

pub mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ibl".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // Bake diffuse radiance and irradiance maps for a given HDR.

    let hdr_paths = [
        Path::new("./examples/ibl/assets/poly_haven_studio_4k.hdr"),
        Path::new("./examples/ibl/assets/kloppenheim_06_puresky_4k.hdr"),
        Path::new("./examples/ibl/assets/rural_asphalt_road_4k.hdr"),
        Path::new("./examples/ibl/assets/thatch_chapel_4k.hdr"),
    ];

    let hdr_path_index_rc = RefCell::new(0_usize);

    // Set up a sphere grid (scene).

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut skybox_arena = resources.skybox.borrow_mut();

        make_sphere_grid_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            &mut texture_u8_arena,
            &mut point_light_arena,
            &mut skybox_arena,
        )
    }?;

    {
        if let Some(skybox_handle) = scene
            .root
            .find(|node| *node.get_type() == SceneNodeType::Skybox)?
        {
            let resources = &scene_context.resources;

            let mut skybox_arena = resources.skybox.borrow_mut();

            if let Ok(entry) = skybox_arena.get_mut(&skybox_handle) {
                let skybox = &mut entry.item;

                let mut texture_vec2_arena = resources.texture_vec2.borrow_mut();
                let mut cubemap_vec3_arena = resources.cubemap_vec3.borrow_mut();

                let hdr_path_index = hdr_path_index_rc.borrow();
                let hdr_path = hdr_paths[*hdr_path_index];

                skybox.load_hdr(&mut texture_vec2_arena, &mut cubemap_vec3_arena, hdr_path);
            }
        }
    }

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let update_node = |_current_world_transform: &Mat4,
                       node: &mut SceneNode,
                       _resources: &SceneResources,
                       app: &App,
                       _mouse_state: &MouseState,
                       _keyboard_state: &KeyboardState,
                       _game_controller_state: &GameControllerState,
                       _shader_context: &mut ShaderContext|
     -> Result<bool, String> {
        let uptime = app.timing_info.uptime_seconds;

        let (node_type, _handle) = (node.get_type(), node.get_handle());

        match node_type {
            SceneNodeType::Entity => {
                let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

                node.get_transform_mut().set_rotation(q);

                Ok(false)
            }
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

        let was_num0_pressed = keyboard_state
            .newly_pressed_keycodes
            .contains(&Keycode::Num0);

        let was_num9_pressed = keyboard_state
            .newly_pressed_keycodes
            .contains(&Keycode::Num9);

        if was_num0_pressed || was_num9_pressed {
            let mut current_index = hdr_path_index_rc.borrow_mut();

            *current_index = if was_num0_pressed {
                (*current_index + 1) % hdr_paths.len()
            } else if *current_index == 0 {
                hdr_paths.len() - 1
            } else {
                *current_index - 1
            };

            let hdr_path = hdr_paths[*current_index];

            update_skybox_handles(resources, scene, hdr_path);
        }

        let mut renderer = renderer_rc.borrow_mut();

        renderer.options.update(keyboard_state);

        renderer.shader_options.update(keyboard_state);

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        // Render scene.

        scene.render(resources, &renderer_rc, None)?;

        // Write out.

        let framebuffer = framebuffer_rc.borrow();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let color_buffer = color_buffer_lock.borrow();

                color_buffer.copy_to(canvas);

                Ok(())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn update_skybox_handles(resources: &SceneResources, scene: &mut SceneGraph, hdr_path: &Path) {
    for node in scene.root.children_mut().as_mut().unwrap() {
        if node.is_type(SceneNodeType::Environment) {
            for child in node.children_mut().as_mut().unwrap() {
                if child.is_type(SceneNodeType::Skybox) {
                    let skybox_handle = child.get_handle().unwrap();

                    if let Ok(skybox_entry) = resources.skybox.borrow_mut().get_mut(&skybox_handle)
                    {
                        let skybox = &mut skybox_entry.item;

                        let mut texture_vec2_arena = resources.texture_vec2.borrow_mut();

                        let mut cubemap_vec3_arena = resources.cubemap_vec3.borrow_mut();

                        skybox.load_hdr(&mut texture_vec2_arena, &mut cubemap_vec3_arena, hdr_path);
                    }
                }
            }
        }
    }
}
