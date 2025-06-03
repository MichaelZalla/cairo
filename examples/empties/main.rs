extern crate sdl2;

use std::{cell::RefCell, env, f32::consts::TAU, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1600_BY_900},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
    matrix::Mat4,
    render::Renderer,
    scene::{
        context::SceneContext,
        graph::options::SceneGraphRenderOptions,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3,
};

use scene::make_scene;

pub mod scene;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example render-text /path/to/your-font.fon");
        return Ok(());
    }

    // App

    let mut window_info = AppWindowInfo {
        title: "examples/empties".to_string(),
        canvas_resolution: RESOLUTION_1600_BY_900,
        window_resolution: RESOLUTION_1600_BY_900,
        ..Default::default()
    };

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // Load a system font

    let font_info = FontInfo {
        filepath: args[1].to_string(),
        point_size: 16,
    };

    let font_cache = Box::leak(Box::new(FontCache::new(app.context.ttf_context)));

    let text_cache = TextCache::default();

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::from(&window_info);

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Scene context

    let scene_context = SceneContext::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut point_light_arena = resources.point_light.borrow_mut();
        let mut spot_light_arena = resources.spot_light.borrow_mut();
        let mut empty_arena = resources.empty.borrow_mut();

        make_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut point_light_arena,
            &mut spot_light_arena,
            &mut empty_arena,
        )?
    };

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer =
        SoftwareRenderer::new(shader_context_rc.clone(), scene_context.resources.clone());

    renderer.font_info.replace(font_info);
    renderer.font_cache.replace(font_cache.clone());
    renderer.text_cache.replace(text_cache);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

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

        match node.get_type() {
            SceneNodeType::Empty => {
                let rotate_x = Quaternion::new(vec3::RIGHT, uptime % TAU);
                let rotate_y = Quaternion::new(vec3::UP, uptime % TAU);
                let rotate_z = Quaternion::new(vec3::FORWARD, uptime % TAU);

                let q = rotate_x * rotate_y * rotate_z;

                node.get_transform_mut().set_rotation(q);

                Ok(true)
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

            renderer.render_ground_plane(16, None);
        }

        // Render scene.

        scene.render(
            resources,
            &renderer_rc,
            Some(SceneGraphRenderOptions {
                draw_lights: true,
                draw_node_labels: true,
                ..Default::default()
            }),
        )?;

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.end_frame();
        }

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
