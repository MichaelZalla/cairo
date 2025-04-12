extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1600_BY_900},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    render::Renderer,
    scene::{
        context::{utils::make_empty_scene, SceneContext},
        empty::{Empty, EmptyDisplayKind},
        graph::options::SceneGraphRenderOptions,
        light::{attenuation, point_light::PointLight, spot_light::SpotLight},
        node::{SceneNode, SceneNodeType},
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    transform::Transform3D,
    vec::vec3::Vec3,
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/empties".to_string(),
        relative_mouse_mode: true,
        canvas_resolution: RESOLUTION_1600_BY_900,
        window_resolution: RESOLUTION_1600_BY_900,
        ..Default::default()
    };

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

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

        let (mut scene, shader_context) = make_empty_scene(
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
        )?;

        // Add various "empties" to our scene.

        static EMPTY_DISPLAY_KINDS: [EmptyDisplayKind; 6] = [
            EmptyDisplayKind::Axes,
            EmptyDisplayKind::Arrow,
            EmptyDisplayKind::Square,
            EmptyDisplayKind::Cube,
            EmptyDisplayKind::Circle(36),
            EmptyDisplayKind::Sphere(36),
        ];

        for (index, kind) in EMPTY_DISPLAY_KINDS.iter().enumerate() {
            let empty_node = {
                let empty = Empty(*kind);

                let empty_handle = empty_arena.insert(empty);

                let mut transform = Transform3D::default();

                transform.set_translation(Vec3 {
                    x: (EMPTY_DISPLAY_KINDS.len() as f32 * -2.0) + index as f32 * 4.0,
                    y: 2.0,
                    z: -0.5,
                });

                SceneNode::new(SceneNodeType::Empty, transform, Some(empty_handle))
            };

            scene.root.add_child(empty_node)?;
        }

        // Add a point light to our scene.

        let point_light_node = {
            let mut point_light = PointLight::new();

            point_light.intensities = color::RED.to_vec3() / 255.0;

            point_light.set_attenuation(attenuation::LIGHT_ATTENUATION_RANGE_13_UNITS);

            let point_light_handle = point_light_arena.insert(point_light);

            let mut transform = Transform3D::default();

            transform.set_translation(Vec3 {
                x: -6.0,
                y: 8.0,
                z: -6.0,
            });

            SceneNode::new(
                SceneNodeType::PointLight,
                transform,
                Some(point_light_handle),
            )
        };

        scene.root.add_child(point_light_node)?;

        // Add a spot light to our scene.

        let spot_light_node = {
            let mut spot_light = SpotLight::new();

            spot_light.intensities = color::YELLOW.to_vec3() / 255.0 * 2.0;

            spot_light.set_attenuation(attenuation::LIGHT_ATTENUATION_RANGE_20_UNITS);

            let spot_light_handle = spot_light_arena.insert(spot_light);

            let mut transform = Transform3D::default();

            transform.set_translation(Vec3 {
                x: 6.0,
                y: 8.0,
                z: 6.0,
            });

            SceneNode::new(SceneNodeType::SpotLight, transform, Some(spot_light_handle))
        };

        scene.root.add_child(spot_light_node)?;

        (scene, shader_context)
    };

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
        Default::default(),
    );

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // App update and render callbacks

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

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            None,
        )?;

        let mut renderer = renderer_rc.borrow_mut();

        renderer.options.update(keyboard_state);

        renderer.shader_options.update(keyboard_state);

        let camera_handle = scene
            .root
            .find(|node| *node.get_type() == SceneNodeType::Camera)
            .unwrap()
            .unwrap();

        let camera_arena = resources.camera.borrow();

        if let Ok(entry) = camera_arena.get(&camera_handle) {
            let camera = &entry.item;

            renderer.set_clipping_frustum(*camera.get_frustum());
        }

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

            renderer.render_ground_plane(16);
        }

        // Render scene.

        scene.render(
            resources,
            &renderer_rc,
            Some(SceneGraphRenderOptions {
                draw_lights: true,
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
