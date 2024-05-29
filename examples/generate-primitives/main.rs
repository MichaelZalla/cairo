extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use sdl2::keyboard::Keycode;

use uuid::Uuid;

use cairo::{
    app::{resolution::RESOLUTIONS_16X9, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    debug::message::DebugMessageBuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    entity::Entity,
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
    material::Material,
    matrix::Mat4,
    mesh,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        light::{PointLight, SpotLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shader::context::ShaderContext,
    shaders::{
        debug_shaders::{
            albedo_fragment_shader::AlbedoFragmentShader,
            depth_fragment_shader::DepthFragmentShader,
            normal_fragment_shader::NormalFragmentShader,
            uv_test_fragment_shader::UvTestFragmentShader,
        },
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::{zbuffer::DepthTestMethod, SoftwareRenderer},
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    vec::vec3::Vec3,
};

fn main() -> Result<(), String> {
    let current_resolution_index_rc: RefCell<usize> = RefCell::new(2);

    let resolution = RESOLUTIONS_16X9[*current_resolution_index_rc.borrow()];

    let mut window_info = AppWindowInfo {
        title: "examples/generate-primitives".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        window_resolution: resolution,
        canvas_resolution: resolution,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Fonts

    let font_info = Box::leak(Box::new(FontInfo {
        filepath: "C:/Windows/Fonts/vgasys.fon".to_string(),
        point_size: 16,
    }));

    let font_cache_rc = Box::leak(Box::new(RefCell::new(FontCache::new(
        app.context.ttf_context,
    ))));

    font_cache_rc.borrow_mut().load(font_info)?;

    // Debug messages

    let debug_message_buffer_rc: RefCell<DebugMessageBuffer> = Default::default();

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    static LIGHT_GRID_SUBDIVISIONS: usize = 1;
    static LIGHT_GRID_SIZE: f32 = 20.0;
    static POINT_LIGHTS_COUNT: usize = (LIGHT_GRID_SUBDIVISIONS + 1).pow(2);

    let scene_context = Rc::new(make_empty_scene(framebuffer_rc.borrow().width_over_height)?);

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a textured ground plane to our scene.

        {
            let mut materials = resources.material.borrow_mut();

            let checkerboard_material = {
                let mut material = Material::new("checkerboard".to_string());

                let mut albedo_map = TextureMap::new(
                    "./assets/textures/checkerboard.jpg",
                    TextureMapStorageFormat::Index8(0),
                );

                // Checkerboard material

                albedo_map.sampling_options.wrapping = TextureMapWrapping::Repeat;

                albedo_map.load(rendering_context)?;

                // Pump up albedo value of the darkest pixels

                albedo_map.map(|r, g, b| {
                    if r < 4 && g < 4 && b < 4 {
                        return (18, 18, 18);
                    }
                    (r, g, b)
                })?;

                let albedo_map_handle = resources
                    .texture_u8
                    .borrow_mut()
                    .insert(Uuid::new_v4(), albedo_map);

                material.albedo_map = Some(albedo_map_handle);

                material
            };

            materials.insert(checkerboard_material);
        }

        let plane_entity_node = {
            let mut mesh = mesh::primitive::plane::generate(32.0, 32.0, 1, 1);

            mesh.material_name = Some("checkerboard".to_string());

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: -5.0,
                z: -5.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(plane_entity_node)?;

        // Add a cube to our scene.

        let cube_entity_node = {
            let mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: -4.0,
                y: 1.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(cube_entity_node)?;

        // Add a cone to our scene.

        let cone_entity_node = {
            let mesh = mesh::primitive::cone::generate(2.0, 2.0, 40);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: 0.0,
                y: 1.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(cone_entity_node)?;

        // Add a cylinder to our scene.

        let cylinder_entity_node = {
            let mesh = mesh::primitive::cylinder::generate(2.0, 2.0, 40);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: 4.0,
                y: 1.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(cylinder_entity_node)?;

        // Add point lights to our scene.

        let point_light_decal_material = {
            let mut material = Material::new("point_light_decal".to_string());

            material.alpha_map = Some(resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new(
                    "./assets/decals/point_light_small.png",
                    TextureMapStorageFormat::Index8(0),
                ),
            ));

            material.emissive_color_map = material.alpha_map;

            material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

            material
        };

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(point_light_decal_material);
        }

        {
            let mut point_light_arena = resources.point_light.borrow_mut();

            for x in 0..(LIGHT_GRID_SUBDIVISIONS + 1) {
                for z in 0..(LIGHT_GRID_SUBDIVISIONS + 1) {
                    let mut light = PointLight::new();

                    light.position = Vec3 {
                        x: -(LIGHT_GRID_SIZE / 2.0)
                            + (x as f32 / LIGHT_GRID_SUBDIVISIONS as f32) * LIGHT_GRID_SIZE,
                        y: 1.0,
                        z: -(LIGHT_GRID_SIZE / 2.0)
                            + (z as f32 / LIGHT_GRID_SUBDIVISIONS as f32) * LIGHT_GRID_SIZE,
                    };

                    let point_light_handle = point_light_arena.insert(Uuid::new_v4(), light);

                    let point_light_node = SceneNode::new(
                        SceneNodeType::PointLight,
                        Default::default(),
                        Some(point_light_handle),
                    );

                    scene.root.add_child(point_light_node)?;
                }
            }
        }

        // Add a spot light to our scene.

        let spot_light_decal_material = {
            let mut material = Material::new("spot_light_decal".to_string());

            material.alpha_map = Some(resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new(
                    "./assets/decals/spot_light_small.png",
                    TextureMapStorageFormat::Index8(0),
                ),
            ));

            material.emissive_color_map = material.alpha_map;

            material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

            material
        };

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(spot_light_decal_material);
        }

        let spot_light_node = {
            let mut spot_light: SpotLight = SpotLight::new();

            spot_light.look_vector.set_position(Vec3 {
                x: -6.0,
                y: 15.0,
                z: -6.0,
            });

            let spot_light_handle = resources
                .spot_light
                .borrow_mut()
                .insert(Uuid::new_v4(), spot_light);

            SceneNode::new(
                SceneNodeType::SpotLight,
                Default::default(),
                Some(spot_light_handle),
            )
        };

        scene.root.add_child(spot_light_node)?;

        // Add a second camera to our scene.

        let camera_node = {
            let camera: Camera = Camera::from_perspective(
                Vec3 {
                    x: 0.0,
                    y: 12.0,
                    z: -16.0,
                },
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.5,
                }
                .as_normal(),
                75.0,
                framebuffer_rc.borrow().width_over_height,
            );

            let camera_handle = resources.camera.borrow_mut().insert(Uuid::new_v4(), camera);

            SceneNode::new(
                SceneNodeType::Camera,
                Default::default(),
                Some(camera_handle),
            )
        };

        scene.root.add_child(camera_node)?;
    }

    // Shader context

    let shader_context_rc: Rc<RefCell<ShaderContext>> = Default::default();

    // Fragment shaders

    let fragment_shaders = [
        DEFAULT_FRAGMENT_SHADER,
        AlbedoFragmentShader,
        DepthFragmentShader,
        NormalFragmentShader,
        UvTestFragmentShader,
    ];

    let active_fragment_shader_index_rc: RefCell<usize> = Default::default();

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    let looking_at_point_light_rc = RefCell::new(false);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let mut renderer = renderer_rc.borrow_mut();

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::R { .. } => {
                    // Resize the app's rendering canvas.

                    let mut current_resolution_index = current_resolution_index_rc.borrow_mut();

                    *current_resolution_index =
                        (*current_resolution_index + 1) % RESOLUTIONS_16X9.len();

                    let new_resolution = RESOLUTIONS_16X9[*current_resolution_index];

                    app.resize_window(new_resolution).unwrap();

                    app.resize_canvas(new_resolution).unwrap();

                    // Resize the framebuffer to match.
                    let mut framebuffer = framebuffer_rc.borrow_mut();

                    framebuffer.resize(new_resolution.width, new_resolution.height, true);
                }
                Keycode::H { .. } => {
                    let mut active_fragment_shader_index =
                        active_fragment_shader_index_rc.borrow_mut();

                    *active_fragment_shader_index += 1;

                    if *active_fragment_shader_index == fragment_shaders.len() {
                        *active_fragment_shader_index = 0;
                    }

                    // let mut renderer = renderer_rc.borrow_mut();

                    renderer.set_fragment_shader(fragment_shaders[*active_fragment_shader_index]);
                }
                Keycode::L { .. } => {
                    let mut looking_at_point_light = looking_at_point_light_rc.borrow_mut();

                    *looking_at_point_light = !*looking_at_point_light;
                }
                _ => {}
            }
        }

        let mut debug_message_buffer = debug_message_buffer_rc.borrow_mut();

        debug_message_buffer.write(format!(
            "Resolution: {}x{}",
            app.window_info.canvas_resolution.width, app.window_info.canvas_resolution.height
        ));

        let uptime = app.timing_info.uptime_seconds;

        debug_message_buffer.write(format!("FPS: {:.*}", 0, app.timing_info.frames_per_second));

        debug_message_buffer.write(format!("Seconds ellapsed: {:.*}", 2, uptime));

        let resources = scene_context.resources.borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context.set_ambient_light(None);
        shader_context.set_directional_light(None);
        shader_context.get_point_lights_mut().clear();
        shader_context.get_spot_lights_mut().clear();

        // Traverse the scene graph and update its nodes.

        static COLOR_CHANNEL_PHASE_SHIFT: f32 = 2.0 * PI / 3.0;

        let mut point_lights_visited: usize = 0;
        let mut spot_lights_visited: usize = 0;

        let mut update_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &mut SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mut entity_arena = resources.entity.borrow_mut();

                        match entity_arena.get_mut(handle) {
                            Ok(entry) => {
                                let entity = &mut entry.item;

                                if let Ok(entry) = resources.mesh.borrow_mut().get(&entity.mesh) {
                                    let mesh = &entry.item;

                                    if let Some(object_name) = &mesh.object_name {
                                        if object_name == "plane" {
                                            return Ok(());
                                        }
                                    }
                                }

                                static ENTITY_ROTATION_SPEED: f32 = 0.3;

                                let mut rotation = *node.get_transform().rotation();

                                rotation.z += 1.0
                                    * ENTITY_ROTATION_SPEED
                                    * PI
                                    * app.timing_info.seconds_since_last_update;

                                rotation.z %= 2.0 * PI;

                                rotation.x += 1.0
                                    * ENTITY_ROTATION_SPEED
                                    * PI
                                    * app.timing_info.seconds_since_last_update;

                                rotation.x %= 2.0 * PI;

                                rotation.y += 1.0
                                    * ENTITY_ROTATION_SPEED
                                    * PI
                                    * app.timing_info.seconds_since_last_update;

                                rotation.y %= 2.0 * PI;

                                node.get_transform_mut().set_rotation(rotation);

                                Ok(())
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
                SceneNodeType::Camera => {
                    match handle {
                        Some(handle) => {
                            let mut camera_arena = resources.camera.borrow_mut();

                            match camera_arena.get_mut(handle) {
                                Ok(entry) => {
                                    let camera = &mut entry.item;

                                    debug_message_buffer.write(format!(
                                        "Camera position: {}",
                                        camera.look_vector.get_position()
                                    ));
                                }
                                Err(err) => panic!(
                                    "Failed to get Camera from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered a `Camera` node with no resource handle!")
                        }
                    }

                    node.update(
                        &current_world_transform,
                        &resources,
                        app,
                        mouse_state,
                        keyboard_state,
                        game_controller_state,
                        &mut shader_context,
                    )
                }
                SceneNodeType::PointLight => match handle {
                    Some(handle) => {
                        let mut arena = resources.point_light.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                static LIGHT_SPEED_FACTOR: f32 = 0.66;
                                static ORBIT_RADIUS: f32 = 12.0;
                                static MAX_POINT_LIGHT_INTENSITY: f32 = 25.0;

                                let light_phase_shift = (2.0 * PI / (POINT_LIGHTS_COUNT as f32))
                                    * point_lights_visited as f32;

                                light.intensities = Vec3 {
                                    x: (uptime
                                        + COLOR_CHANNEL_PHASE_SHIFT * 0.0
                                        + light_phase_shift)
                                        .sin()
                                        / 2.0
                                        + 0.5,
                                    y: (uptime
                                        + COLOR_CHANNEL_PHASE_SHIFT * 1.0
                                        + light_phase_shift)
                                        .sin()
                                        / 2.0
                                        + 0.5,
                                    z: (uptime
                                        + COLOR_CHANNEL_PHASE_SHIFT * 2.0
                                        + light_phase_shift)
                                        .sin()
                                        / 2.0
                                        + 0.5,
                                } * MAX_POINT_LIGHT_INTENSITY;

                                let offset = point_lights_visited % 2 == 0;

                                light.position = Vec3 {
                                    x: ORBIT_RADIUS
                                        * ((uptime * LIGHT_SPEED_FACTOR) + light_phase_shift).sin()
                                        * if offset { 1.5 } else { 1.0 },
                                    y: 1.0,
                                    z: ORBIT_RADIUS
                                        * ((uptime * LIGHT_SPEED_FACTOR) + light_phase_shift).cos()
                                        * if offset { 1.5 } else { 1.0 },
                                };

                                shader_context.get_point_lights_mut().push(*handle);

                                point_lights_visited += 1;

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get PointLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `PointLight` node with no resource handle!")
                    }
                },
                SceneNodeType::SpotLight => match handle {
                    Some(handle) => {
                        let mut arena = resources.spot_light.borrow_mut();

                        match arena.get_mut(handle) {
                            Ok(entry) => {
                                let light = &mut entry.item;

                                static MAX_SPOT_LIGHT_INTENSITY: f32 = 25.0;

                                light.intensities = Vec3 {
                                    x: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 0.0).cos() / 2.0 + 0.5,
                                    y: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 1.0).cos() / 2.0 + 0.5,
                                    z: (uptime + COLOR_CHANNEL_PHASE_SHIFT * 2.0).cos() / 2.0 + 0.5,
                                } * MAX_SPOT_LIGHT_INTENSITY;

                                shader_context.get_spot_lights_mut().push(*handle);

                                spot_lights_visited += 1;

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get SpotLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `SpotLight` node with no resource handle!")
                    }
                },
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

        renderer
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        debug_message_buffer.write(format!(
            "Wireframe: {}",
            if renderer.options.do_wireframe {
                "On"
            } else {
                "Off"
            }
        ));

        debug_message_buffer.write(format!(
            "Rasterized geometry: {}",
            if renderer.options.do_rasterized_geometry {
                "On"
            } else {
                "Off"
            }
        ));

        if renderer.options.do_rasterized_geometry {
            debug_message_buffer.write(format!(
                "Culling reject mask: {:?}",
                renderer.options.face_culling_strategy.reject
            ));

            debug_message_buffer.write(format!(
                "Culling window order: {:?}",
                renderer.options.face_culling_strategy.winding_order
            ));

            {
                let framebuffer = framebuffer_rc.borrow();

                let depth_buffer = framebuffer.attachments.depth.as_ref().unwrap().borrow();

                debug_message_buffer.write(format!(
                    "Depth test method: {:?}",
                    depth_buffer.get_depth_test_method()
                ));
            }

            debug_message_buffer.write(format!(
                "Lighting: {}",
                if renderer.options.do_lighting {
                    "On"
                } else {
                    "Off"
                }
            ));

            renderer
                .shader_options
                .update(keyboard_state, mouse_state, game_controller_state);

            //

            let mut active_fragment_shader_index = active_fragment_shader_index_rc.borrow_mut();

            for keycode in &keyboard_state.keys_pressed {
                match keycode {
                    Keycode::I { .. } => {
                        let framebuffer = framebuffer_rc.borrow_mut();

                        let mut depth_buffer =
                            framebuffer.attachments.depth.as_ref().unwrap().borrow_mut();

                        let methods = [
                            DepthTestMethod::Always,
                            DepthTestMethod::Never,
                            DepthTestMethod::Less,
                            DepthTestMethod::Equal,
                            DepthTestMethod::LessThanOrEqual,
                            DepthTestMethod::Greater,
                            DepthTestMethod::NotEqual,
                            DepthTestMethod::GreaterThanOrEqual,
                        ];

                        let mut index = methods
                            .iter()
                            .position(|&method| method == *(depth_buffer.get_depth_test_method()))
                            .unwrap();

                        index = if index == (methods.len() - 1) {
                            0
                        } else {
                            index + 1
                        };

                        depth_buffer.set_depth_test_method(methods[index])
                    }
                    Keycode::H { .. } => {
                        *active_fragment_shader_index += 1;

                        if *active_fragment_shader_index == fragment_shaders.len() {
                            *active_fragment_shader_index = 0;
                        }

                        renderer
                            .set_fragment_shader(fragment_shaders[*active_fragment_shader_index]);
                    }
                    _ => {}
                }
            }

            debug_message_buffer.write(format!(
                "Fragment shader: {}",
                [
                    "DEFAULT_FRAGMENT_SHADER",
                    "AlbedoFragmentShader",
                    "DepthFragmentShader",
                    "NormalFragmentShader",
                    "UvTestFragmentShader",
                ][*active_fragment_shader_index]
            ));
        }

        debug_message_buffer.write(format!(
            "Visualize normals: {}",
            if renderer.options.do_visualize_normals {
                "On"
            } else {
                "Off"
            }
        ));

        debug_message_buffer.write(format!(
            "Looking at point light: {}",
            looking_at_point_light_rc.borrow(),
        ));

        Ok(())
    };

    let mut render = |_frame_index| -> Result<Vec<u32>, String> {
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
                        let mut color_buffer = color_buffer_lock.borrow_mut();

                        {
                            let debug_messages = &mut *debug_message_buffer_rc.borrow_mut();
                            let mut font_cache = font_cache_rc.borrow_mut();

                            Graphics::render_debug_messages(
                                &mut color_buffer,
                                &mut font_cache,
                                font_info,
                                (12, 12),
                                1.0,
                                debug_messages,
                            );
                        }

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
