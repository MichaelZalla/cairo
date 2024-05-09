use std::cell::RefCell;

use scene::make_sphere_grid_scene;

use cairo::{
    app::{resolution::RESOLUTION_1280_BY_720, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
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

pub mod callbacks;
pub mod scene;

use callbacks::{
    render_scene_graph_node_default, render_skybox_node_default, update_scene_graph_node_default,
};

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

    let pipeline = Pipeline::new(
        &shader_context_rc,
        scene_context_rc.borrow().resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    let pipeline_rc = RefCell::new(pipeline);

    // App update() callback

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let scene_context = scene_context_rc.borrow_mut();
        let mut scene_resources = (*(scene_context.resources)).borrow_mut();

        let mut shader_context = shader_context_rc.borrow_mut();

        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        let mut update_scene_graph_node = |_current_depth: usize,
                                           _current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let mut framebuffer = framebuffer_rc.borrow_mut();

            update_scene_graph_node_default(
                &mut framebuffer,
                &mut shader_context,
                &mut scene_resources,
                node,
                &app.timing_info,
                keyboard_state,
                mouse_state,
                game_controller_state,
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
        let mut pipeline = pipeline_rc.borrow_mut();

        pipeline.bind_framebuffer(Some(&framebuffer_rc));

        // Begin frame

        pipeline.begin_frame();

        // Render scene.

        let scene_context = scene_context_rc.borrow();
        let scene_resources = scene_context.resources.borrow();

        let scene = &scene_context.scenes.borrow()[0];

        let mut skybox_handle: Option<Handle> = None;
        let mut camera_handle: Option<Handle> = None;

        let mut render_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &SceneNode|
         -> Result<(), String> {
            render_scene_graph_node_default(
                &mut pipeline,
                &scene_resources,
                node,
                &current_world_transform,
                &mut skybox_handle,
                &mut camera_handle,
            )
        };

        // Traverse the scene graph and render its nodes.

        scene.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut render_scene_graph_node,
        )?;

        // Skybox pass

        if let (Some(camera_handle), Some(skybox_handle)) = (camera_handle, skybox_handle) {
            render_skybox_node_default(
                &mut pipeline,
                &scene_resources,
                &skybox_handle,
                &camera_handle,
            );
        }

        // End frame

        pipeline.end_frame();

        // Write out.

        let framebuffer = framebuffer_rc.borrow();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let color_buffer = color_buffer_lock.borrow();

                Ok(color_buffer.get_all().clone())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
