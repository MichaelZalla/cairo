extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI};

use uuid::Uuid;

use cairo::{
    app::{resolution::RESOLUTION_1200_BY_675, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh,
    pipeline::Pipeline,
    scene::{
        context::utils::make_empty_scene,
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
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::vec3::Vec3,
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/normal-map".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        window_resolution: RESOLUTION_1200_BY_675,
        canvas_resolution: RESOLUTION_1200_BY_675,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = RefCell::new(framebuffer);

    // Scene context

    let scene_context = make_empty_scene(framebuffer_rc.borrow().width_over_height)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Brick wall material.

        let mut brick_material = Material::new("brick".to_string());

        brick_material.albedo_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/normal-map/assets/Brick_OldDestroyed_1k_d.tga",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        brick_material.specular_exponent_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/normal-map/assets/Brick_OldDestroyed_1k_s.tga",
                TextureMapStorageFormat::Index8(0),
            ),
        ));

        brick_material.normal_map = Some(resources.texture_u8.borrow_mut().insert(
            Uuid::new_v4(),
            TextureMap::new(
                "./examples/normal-map/assets/Brick_OldDestroyed_1k_nY+.tga",
                TextureMapStorageFormat::RGB24,
            ),
        ));

        brick_material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(brick_material);
        }

        // Add a brick wall to our scene.

        let brick_wall_mesh = mesh::primitive::cube::generate(1.5, 1.5, 1.5);

        let brick_wall_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_mesh);

        let brick_wall_entity = Entity::new(brick_wall_mesh_handle, Some("brick".to_string()));

        let brick_wall_entity_handle = resources
            .entity
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_entity);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(brick_wall_entity_handle),
        ))?;

        // Add a point light to our scene.

        let point_light_node = {
            let mut light = PointLight::new();

            light.position.y = 0.0;
            light.position.z = -4.0;

            light.intensities = Vec3::ones() * 10.0;

            light.constant_attenuation = 1.0;
            light.linear_attenuation = 0.35;
            light.quadratic_attenuation = 0.44;

            let point_light_handle = resources
                .point_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

            SceneNode::new(
                SceneNodeType::PointLight,
                Default::default(),
                Some(point_light_handle),
            )
        };

        scene.root.add_child(point_light_node)?;

        // Add a spot light to our scene.

        let spot_light_node = {
            let light = SpotLight::new();

            let light_handle = resources
                .spot_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

            SceneNode::new(
                SceneNodeType::SpotLight,
                Default::default(),
                Some(light_handle),
            )
        };

        scene.root.add_child(spot_light_node)?;
    }

    let scene_context_rc = RefCell::new(scene_context);

    // Shader context

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        scene_context_rc.borrow().resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.bind_framebuffer(Some(&framebuffer_rc));

    pipeline.geometry_shader_options.base_color_mapping_active = false;
    pipeline.geometry_shader_options.normal_mapping_active = true;

    let pipeline_rc = RefCell::new(pipeline);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let scene_context = scene_context_rc.borrow_mut();
        let resources = scene_context.resources.borrow_mut();
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
            let (node_type, _handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => {
                    static ENTITY_ROTATION_SPEED: f32 = 0.1;

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
                _ => node.update(
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
        // Render scene.

        let scene_context = scene_context_rc.borrow();
        let resources = (*scene_context.resources).borrow();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        let mut pipeline = pipeline_rc.borrow_mut();

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
