extern crate sdl2;

use core::f32;
use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720, RESOLUTION_960_BY_540},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    geometry::accelerator::static_triangle_bvh::StaticTriangleBVH,
    matrix::Mat4,
    mesh::obj::load::{load_obj, LoadObjResult, ProcessGeometryFlag},
    render::options::RenderOptions,
    resource::handle::Handle,
    scene::{
        context::SceneContext,
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

use scene::make_collision_physics_scene;

mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/collision-physics".to_string(),
        relative_mouse_mode: true,
        canvas_resolution: RESOLUTION_960_BY_540,
        window_resolution: RESOLUTION_1280_BY_720,
        ..Default::default()
    };

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    let rendering_context = &app.context.rendering_context;

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Load some level (collision) geometry.

    let scene_context = SceneContext::default();

    let mut level_meshes = {
        let resources = &scene_context.resources;

        let mut material_arena = resources.material.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        let LoadObjResult(_level_geometry, level_meshes) = load_obj(
            "./data/blender/collision-level/collision-level_004.obj",
            &mut material_arena,
            &mut texture_u8_arena,
            Some(ProcessGeometryFlag::Null | ProcessGeometryFlag::Center),
        );

        level_meshes
    };

    for mesh in level_meshes.iter_mut() {
        mesh.static_triangle_bvh
            .replace(StaticTriangleBVH::new(mesh));
    }

    // Scene context

    let mut level_mesh_handle = Handle::default();

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

        make_collision_physics_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
            &mut texture_u8_arena,
            rendering_context,
            level_meshes,
            &mut level_mesh_handle,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        RenderOptions {
            draw_normals_scale: 0.25,
            ..Default::default()
        },
    );

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let view_camera_handle: &'static RefCell<Option<Handle>> =
        Box::leak(Box::new(RefCell::new(Default::default())));

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

        let update_node_rc = Rc::new(
            |_current_world_transform: &Mat4,
             node: &mut SceneNode,
             resources: &SceneResources,
             app: &App,
             _mouse_state: &MouseState,
             _keyboard_state: &KeyboardState,
             _game_controller_state: &GameControllerState,
             _shader_context: &mut ShaderContext|
             -> Result<bool, String> {
                let uptime = app.timing_info.uptime_seconds;

                let (node_type, handle) = (node.get_type(), node.get_handle());

                match node_type {
                    SceneNodeType::Camera => match handle {
                        Some(handle) => {
                            view_camera_handle.borrow_mut().replace(*handle);

                            Ok(false)
                        }
                        None => panic!("Encountered a `Camera` node with no resource handle!"),
                    },
                    SceneNodeType::DirectionalLight => {
                        if let Ok(entry) = resources
                            .directional_light
                            .borrow_mut()
                            .get_mut(&node.get_handle().unwrap())
                        {
                            let light = &mut entry.item;

                            let rotate_x = Quaternion::new(vec3::RIGHT, -PI / 4.0);

                            let rotate_y =
                                Quaternion::new(vec3::UP, uptime / 2.0 % f32::consts::TAU);

                            light.set_direction(rotate_x * rotate_y);
                        }

                        Ok(false)
                    }
                    _ => Ok(false),
                }
            },
        );

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_node_rc),
        )?;

        {
            let mut camera_arena = resources.camera.borrow_mut();

            for entry in camera_arena.entries.iter_mut().flatten() {
                let camera = &mut entry.item;

                if camera.is_active {
                    let framebuffer = framebuffer_rc.borrow_mut();

                    if let Some(depth_buffer_rc) = framebuffer.attachments.depth.as_ref() {
                        let mut depth_buffer = depth_buffer_rc.borrow_mut();

                        depth_buffer.set_projection_z_near(camera.get_projection_z_near());
                        depth_buffer.set_projection_z_far(camera.get_projection_z_far());
                    }
                }
            }
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
