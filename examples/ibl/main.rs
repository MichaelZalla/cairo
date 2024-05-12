use std::{borrow::BorrowMut, cell::RefCell, path::Path};

use uuid::Uuid;

use sdl2::keyboard::Keycode;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    matrix::Mat4,
    pipeline::Pipeline,
    resource::handle::Handle,
    scene::{
        graph::SceneGraph,
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
};

use bake::{bake_diffuse_and_specular_from_hdri, brdf::generate_specular_brdf_integration_map};

pub mod bake;
pub mod scene;
pub mod shaders;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/ibl".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

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

    let framebuffer_rc = RefCell::new(framebuffer);

    let mut spheres_scene_context =
        scene::make_sphere_grid_scene(framebuffer_rc.borrow().width_over_height).unwrap();

    {
        let scene_context = spheres_scene_context.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        for node in scene.root.children_mut().as_mut().unwrap() {
            if node.is_type(SceneNodeType::Environment) {
                // No handle for now.

                let skybox_node = SceneNode::new(SceneNodeType::Skybox, Default::default(), None);

                node.add_child(skybox_node)?;
            }
        }
    }

    // Generate a common BRDF integration map, approximating the integral formed
    // by our Geometry function over varying angles and roughness values.

    let specular_brdf_integration_map_handle: Handle;

    {
        let resources = (*spheres_scene_context.resources).borrow_mut();
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
        let resources = (*spheres_scene_context.resources).borrow_mut();

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

    let shader_context_rc: RefCell<ShaderContext> = Default::default();

    {
        let mut shader_context = shader_context_rc.borrow_mut();

        shader_context.set_active_ambient_specular_brdf_integration_map(Some(
            specular_brdf_integration_map_handle,
        ));

        set_ibl_map_handles(
            &mut shader_context,
            &mut spheres_scene_context.borrow_mut().scenes.borrow_mut()[0],
            &radiance_irradiance_handles[0],
        );
    }

    let mut pipeline = Pipeline::new(
        &shader_context_rc,
        spheres_scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.bind_framebuffer(Some(&framebuffer_rc));

    let spheres_scene_context_rc = RefCell::new(spheres_scene_context);

    let pipeline_rc = RefCell::new(pipeline);

    // App update and render callbacks

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let scene_context = spheres_scene_context_rc.borrow_mut();

        let resources = (*scene_context.resources).borrow();

        let mut scenes = scene_context.scenes.borrow_mut();
        let scene = &mut scenes[0];

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
            node.update(
                &resources,
                app,
                mouse_state,
                keyboard_state,
                game_controller_state,
                &mut shader_context,
            )
        };

        scene.root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut update_scene_graph_node,
        )?;

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Num0 | Keycode::Num9 => {
                    let mut current_index = current_handles_index.borrow_mut();

                    *current_index = if *keycode == Keycode::Num0 {
                        (*current_index + 1) % radiance_irradiance_handles.len()
                    } else if *current_index == 0 {
                        radiance_irradiance_handles.len() - 1
                    } else {
                        *current_index - 1
                    };

                    println!("{}", current_index);

                    set_ibl_map_handles(
                        &mut shader_context,
                        scene,
                        &radiance_irradiance_handles[*current_index],
                    );
                }
                _ => (),
            }
        }

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
        // Delegate the rendering to our scene.

        // Render scene.

        let scene_context = spheres_scene_context_rc.borrow();
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

fn set_ibl_map_handles(
    shader_context: &mut ShaderContext,
    scene: &mut SceneGraph,
    handles: &(Handle, Handle, Handle),
) {
    let (
        radiance_cubemap_handle,
        irradiance_cubemap_handle,
        specular_prefiltered_environment_cubemap_handle,
    ) = handles;

    shader_context.set_active_ambient_radiance_map(Some(*radiance_cubemap_handle));

    shader_context.set_active_ambient_diffuse_irradiance_map(Some(*irradiance_cubemap_handle));

    shader_context.set_active_ambient_specular_prefiltered_environment_map(Some(
        *specular_prefiltered_environment_cubemap_handle,
    ));

    // Set the irradiance map as our scene's ambient diffuse light map.

    for node in scene.root.children_mut().as_mut().unwrap() {
        if node.is_type(SceneNodeType::Environment) {
            for child in node.children_mut().as_mut().unwrap() {
                if child.is_type(SceneNodeType::Skybox) {
                    child.set_handle(Some(*radiance_cubemap_handle));
                }
            }
        }
    }
}
