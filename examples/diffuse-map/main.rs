extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use uuid::Uuid;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    material::Material,
    matrix::Mat4,
    scene::{
        context::utils::make_cube_scene,
        light::{PointLight, SpotLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::map::TextureMap,
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/diffuse-map".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        ..Default::default()
    };

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Render callback

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let rendering_context = &app.context.rendering_context;

    // Scene context

    let scene_context =
        Rc::new(make_cube_scene(framebuffer_rc.borrow().width_over_height).unwrap());

    {
        let mut resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Customize the cube material.

        let cube_albedo_map_handle = resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./data/obj/cobblestone.png",
                cairo::texture::map::TextureMapStorageFormat::RGB24,
            ),
        );

        let mut cube_material = Material {
            name: "cube".to_string(),
            albedo_map: Some(cube_albedo_map_handle),
            ..Default::default()
        };

        cube_material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

        resources.material.borrow_mut().insert(cube_material);

        let cube_entity_handle = scene
            .root
            .find(&mut |node| *node.get_type() == SceneNodeType::Entity)
            .unwrap()
            .unwrap();

        match resources.entity.get_mut().get_mut(&cube_entity_handle) {
            Ok(entry) => {
                let cube_entity = &mut entry.item;

                cube_entity.material = Some("cube".to_string());
            }
            _ => panic!(),
        }

        // Add a point light to our scene.

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.7;

        point_light.position = Vec3 {
            x: 0.0,
            y: 4.0,
            z: 0.0,
        };

        let point_light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), point_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        ))?;

        // Add a spot light to our scene.

        let spot_light = SpotLight::new();

        let spot_light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), spot_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        ))?;
    }

    // Shader context

    let shader_context_rc: Rc<RefCell<ShaderContext>> = Default::default();

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.normal_mapping_active = true;

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;
        let resources = scene_context.resources.borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        // Traverse the scene graph and update its nodes.

        let mut update_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, _handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => {
                    let rotation_axis = (vec3::UP + vec3::RIGHT) / 2.0;

                    let q = Quaternion::new(rotation_axis, uptime % (2.0 * PI));

                    node.get_transform_mut().set_rotation(q);

                    Ok(())
                }
                _ => node.update(
                    &current_world_transform,
                    &resources,
                    app,
                    mouse_state,
                    keyboard_state,
                    game_controller_state,
                    &mut shader_context,
                ),
            }
        };

        scenes[0].root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut update_scene_graph_node,
        )?;

        let mut renderer = renderer_rc.borrow_mut();

        renderer
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        renderer
            .shader_options
            .update(keyboard_state, mouse_state, game_controller_state);

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> {
        // Render scene.

        let resources = (*scene_context.resources).borrow();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        match scene.render(&resources, &renderer_rc, None) {
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

    app.run(&mut update, &render)?;

    Ok(())
}
