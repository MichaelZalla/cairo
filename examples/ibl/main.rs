use std::{borrow::BorrowMut, cell::RefCell, f32::consts::PI, path::Path, rc::Rc};

use uuid::Uuid;

use sdl2::keyboard::Keycode;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    physics::pbr::bake::{
        bake_diffuse_and_specular_from_hdri, brdf::generate_specular_brdf_integration_map,
    },
    resource::handle::Handle,
    scene::{
        graph::SceneGraph,
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
        resources::SceneResources,
        skybox::Skybox,
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

pub mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ibl".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    // Bake diffuse radiance and irradiance maps for a given HDR.

    let hdr_paths = [
        Path::new("./examples/ibl/assets/poly_haven_studio_4k.hdr"),
        Path::new("./examples/ibl/assets/kloppenheim_06_puresky_4k.hdr"),
        Path::new("./examples/ibl/assets/rural_asphalt_road_4k.hdr"),
        Path::new("./examples/ibl/assets/thatch_chapel_4k.hdr"),
    ];

    // Set up a sphere grid (scene).

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    let (mut scene_context, shader_context) =
        scene::make_sphere_grid_scene(framebuffer_rc.borrow().width_over_height).unwrap();

    {
        let resources = (*scene_context.resources).borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a skybox to our scene.

        for node in scene.root.children_mut().as_mut().unwrap() {
            if node.is_type(SceneNodeType::Environment) {
                let skybox_node = {
                    // No handles for now.
                    let skybox = Skybox {
                        is_hdr: true,
                        radiance: None,
                        irradiance: None,
                        specular_prefiltered_environment: None,
                    };

                    let skybox_handle =
                        resources.skybox.borrow_mut().insert(Uuid::new_v4(), skybox);

                    SceneNode::new(
                        SceneNodeType::Skybox,
                        Default::default(),
                        Some(skybox_handle),
                    )
                };

                node.add_child(skybox_node)?;
            }
        }
    }

    // Generate a common BRDF integration map, approximating the integral formed
    // by our Geometry function over varying angles and roughness values.

    let specular_brdf_integration_map_handle: Handle;

    {
        let resources = (*scene_context.resources).borrow_mut();
        let mut texture_vec2 = resources.texture_vec2.borrow_mut();

        let specular_brdf_integration_map = generate_specular_brdf_integration_map(512);

        specular_brdf_integration_map_handle =
            texture_vec2.insert(Uuid::new_v4(), specular_brdf_integration_map.to_owned());
    }

    // For each HDR image, generate a corresponding radiance-irradiance cubemap
    // pair, and store the textures in our scene's HDR cubemap texture arena.

    let mut radiance_irradiance_handles = vec![];

    let current_handles_index = RefCell::new(0);

    {
        let resources = (*scene_context.resources).borrow_mut();

        let mut cubemap_vec3 = resources.cubemap_vec3.borrow_mut();

        for hdr_path in hdr_paths {
            let bake_result = bake_diffuse_and_specular_from_hdri(hdr_path).unwrap();

            let radiance_cubemap_handle =
                cubemap_vec3.insert(Uuid::new_v4(), bake_result.radiance.to_owned());

            let irradiance_cubemap_handle =
                cubemap_vec3.insert(Uuid::new_v4(), bake_result.diffuse_irradiance.to_owned());

            let specular_prefiltered_environment_cubemap_handle = cubemap_vec3.insert(
                Uuid::new_v4(),
                bake_result.specular_prefiltered_environment.to_owned(),
            );

            radiance_irradiance_handles.push((
                radiance_cubemap_handle,
                irradiance_cubemap_handle,
                specular_prefiltered_environment_cubemap_handle,
            ));
        }
    }

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    {
        let scene_context = scene_context.borrow_mut();

        let mut resources = (*scene_context.resources).borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

        let mut shader_context = (*shader_context_rc).borrow_mut();

        shader_context
            .set_ambient_specular_brdf_integration_map(Some(specular_brdf_integration_map_handle));

        set_ibl_map_handles(&mut resources, scene, &radiance_irradiance_handles[0]);
    }

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

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let mut resources = (*scene_context.resources).borrow_mut();
        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

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
            match node.get_type() {
                SceneNodeType::Skybox => {
                    let uptime = app.timing_info.uptime_seconds;

                    static ROTATION_AXIS: Vec3 = vec3::UP;

                    let q = Quaternion::new(ROTATION_AXIS, (uptime / 3.0) % (2.0 * PI));

                    node.get_transform_mut().set_rotation(q);

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

        scene.root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut update_scene_graph_node,
        )?;

        for keycode in &keyboard_state.keys_pressed {
            if let (Keycode::Num0 | Keycode::Num9, _) = keycode {
                let mut current_index = current_handles_index.borrow_mut();

                *current_index = if keycode.0 == Keycode::Num0 {
                    (*current_index + 1) % radiance_irradiance_handles.len()
                } else if *current_index == 0 {
                    radiance_irradiance_handles.len() - 1
                } else {
                    *current_index - 1
                };

                println!("{}", current_index);

                set_ibl_map_handles(
                    &mut resources,
                    scene,
                    &radiance_irradiance_handles[*current_index],
                );
            }
        }

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

fn set_ibl_map_handles(
    resources: &mut SceneResources,
    scene: &mut SceneGraph,
    handles: &(Handle, Handle, Handle),
) {
    let (
        radiance_cubemap_handle,
        irradiance_cubemap_handle,
        specular_prefiltered_environment_cubemap_handle,
    ) = handles;

    // Updates our skybox node with the current set of IBL maps.

    for node in scene.root.children_mut().as_mut().unwrap() {
        if node.is_type(SceneNodeType::Environment) {
            for child in node.children_mut().as_mut().unwrap() {
                if child.is_type(SceneNodeType::Skybox) {
                    let skybox_handle = child.get_handle().unwrap();

                    if let Ok(skybox_entry) = resources.skybox.borrow_mut().get_mut(&skybox_handle)
                    {
                        let skybox = &mut skybox_entry.item;

                        skybox.radiance = Some(*radiance_cubemap_handle);
                        skybox.irradiance = Some(*irradiance_cubemap_handle);
                        skybox.specular_prefiltered_environment =
                            Some(*specular_prefiltered_environment_cubemap_handle);
                    }
                }
            }
        }
    }
}
