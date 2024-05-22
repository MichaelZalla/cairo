use std::cell::RefCell;

use scene::make_sphere_grid_scene;

use cairo::{
    app::{resolution::RESOLUTION_1280_BY_720, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    matrix::Mat4,
    pipeline::Pipeline,
    scene::node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod},
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
};

pub mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/pbr".to_string(),
        window_resolution: RESOLUTION_1280_BY_720,
        canvas_resolution: RESOLUTION_1280_BY_720,
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let scene_context = make_sphere_grid_scene(16.0 / 9.0)?;

    let scene_context_rc = RefCell::new(scene_context);

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // ShaderContext

    let shader_context: ShaderContext = Default::default();

    let shader_context_rc = RefCell::new(shader_context);

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        scene_context_rc.borrow().resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.bind_framebuffer(Some(&framebuffer_rc));

    let pipeline_rc = RefCell::new(pipeline);

    // App update() callback

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let scene_context = scene_context_rc.borrow_mut();
        let resources = (*(scene_context.resources)).borrow_mut();

        let mut shader_context = shader_context_rc.borrow_mut();

        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

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

        scene_context.scenes.borrow_mut()[0].root.visit_mut(
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

    // App render() callback

    let mut render = || -> Result<Vec<u32>, String> {
        // Render scene.

        let scene_context = scene_context_rc.borrow();
        let resources = (*scene_context.resources).borrow();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        let mut pipeline = pipeline_rc.borrow_mut();

        match scene.render(&resources, &mut pipeline, None) {
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
