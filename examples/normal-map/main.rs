extern crate sdl2;

use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use uuid::Uuid;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1200_BY_675},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh,
    scene::{
        context::utils::make_empty_scene,
        light::{PointLight, SpotLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    texture::map::{TextureMap, TextureMapStorageFormat},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/normal-map".to_string(),
        vertical_sync: true,
        relative_mouse_mode: true,
        window_resolution: RESOLUTION_1200_BY_675,
        canvas_resolution: RESOLUTION_1200_BY_675,
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    // Scene context

    let (scene_context, shader_context) =
        make_empty_scene(framebuffer_rc.borrow().width_over_height)?;

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

        let brick_material_handle = {
            let mut materials = resources.material.borrow_mut();

            materials.insert(Uuid::new_v4(), brick_material)
        };

        // Add a brick wall to our scene.

        let brick_wall_mesh = mesh::primitive::cube::generate(1.5, 1.5, 1.5);

        let brick_wall_mesh_handle = resources
            .mesh
            .borrow_mut()
            .insert(Uuid::new_v4(), brick_wall_mesh);

        let brick_wall_entity = Entity::new(brick_wall_mesh_handle, Some(brick_material_handle));

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

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    renderer.shader_options.base_color_mapping_active = false;
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

        renderer.options.update(keyboard_state);

        renderer
            .shader_options
            .update(keyboard_state);

        Ok(())
    };

    let render = |_frame_index, _new_resolution| -> Result<Vec<u32>, String> {
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
