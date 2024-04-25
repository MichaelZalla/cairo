use std::cell::RefCell;

use uuid::Uuid;

use cairo::{
    app::{resolution::RESOLUTION_1280_BY_720, App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::obj::load::load_obj,
    pipeline::Pipeline,
    resource::handle::Handle,
    scene::{
        camera::Camera,
        context::SceneContext,
        graph::SceneGraph,
        light::{AmbientLight, DirectionalLight, PointLight},
        node::{
            SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
        },
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    transform::Transform3D,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

pub mod callbacks;

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

    let rendering_context = &app.context.rendering_context;

    let scene_context: SceneContext = Default::default();

    // Add scene resources.

    let mut scene = SceneGraph::new();

    {
        let resources = (*(scene_context.resources)).borrow_mut();

        // Populate our scene (graph).

        let ambient_light_node = SceneNode::new(
            SceneNodeType::AmbientLight,
            Default::default(),
            Some(resources.ambient_light.borrow_mut().insert(
                Uuid::new_v4(),
                AmbientLight {
                    intensities: Vec3::ones() * 0.15,
                },
            )),
        );

        let directional_light_node = SceneNode::new(
            SceneNodeType::DirectionalLight,
            Default::default(),
            Some(resources.directional_light.borrow_mut().insert(
                Uuid::new_v4(),
                DirectionalLight {
                    intensities: Vec3::ones() * 0.15,
                    direction: Vec4::new(vec3::FORWARD, 1.0),
                },
            )),
        );

        let mut perspective_camera = Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -16.0,
            },
            Default::default(),
            75.0,
            16.0 / 9.0,
        );

        perspective_camera.movement_speed = 2.0;

        let camera_node = SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(
                resources
                    .camera
                    .borrow_mut()
                    .insert(Uuid::new_v4(), perspective_camera),
            ),
        );

        let mut environment_node =
            SceneNode::new(SceneNodeType::Environment, Default::default(), None);

        environment_node.add_child(ambient_light_node)?;
        environment_node.add_child(directional_light_node)?;

        scene.root.add_child(environment_node)?;
        scene.root.add_child(camera_node)?;

        // Generate a 2x2 grid of point lights.

        for grid_index_x in 0..4 {
            let mut light = PointLight::new();

            light.position = Vec3 {
                x: -8.0 + 4.0 * grid_index_x as f32,
                y: 4.0,
                z: -3.0,
            };

            light.intensities = Vec3::ones() * 15.0;
            light.specular_intensity = 20.0;

            light.constant_attenuation = 1.0;
            light.linear_attenuation = 0.09;
            light.quadratic_attenuation = 0.032;

            let point_light_handle = resources
                .point_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

            let point_light_node = SceneNode::new(
                SceneNodeType::PointLight,
                Default::default(),
                Some(point_light_handle),
            );

            scene.root.add_child(point_light_node)?;
        }

        //

        let result = load_obj(
            "./examples/pbr/assets/sphere.obj",
            &mut resources.texture.borrow_mut(),
        );

        let _geometry = result.0;
        let meshes = result.1;
        let mut materials = result.2;

        {
            let mut texture_arena = resources.texture.borrow_mut();

            for material in &mut materials.as_mut().unwrap().values_mut() {
                material.load_all_maps(&mut texture_arena, rendering_context)?;
            }
        }

        let mesh = meshes[1].to_owned();

        let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

        // Generate a grid of mesh instances.

        static GRID_ROWS: usize = 6;
        static GRID_COLUMNS: usize = 6;
        static SPACING: f32 = 1.0;

        static GRID_HEIGHT: f32 = GRID_ROWS as f32 + (GRID_ROWS as f32 - 1.0) * SPACING;
        static GRID_WIDTH: f32 = GRID_COLUMNS as f32 + (GRID_COLUMNS as f32 - 1.0) * SPACING;

        let base_transform: Transform3D = Default::default();

        for grid_index_y in 0..GRID_ROWS {
            let alpha_y = grid_index_y as f32 / (GRID_ROWS as f32 - 1.0);

            for grid_index_x in 0..GRID_COLUMNS {
                let alpha_x = grid_index_x as f32 / (GRID_COLUMNS as f32 - 1.0);

                let (material_name, material) = {
                    let name = format!("instance_x{}_y{}", grid_index_x, grid_index_y).to_string();

                    (
                        name.clone(),
                        Material {
                            name,
                            albedo: color::RED.to_vec3() / 255.0,
                            roughness: (alpha_x * 0.75).max(0.075),
                            metallic: alpha_y,
                            sheen: 0.0,
                            clearcoat_thickness: 0.0,
                            clearcoat_roughness: 0.0,
                            anisotropy: 0.0,
                            anisotropy_rotation: 0.0,
                            ..Default::default()
                        },
                    )
                };

                resources.material.borrow_mut().insert(material);

                let entity = Entity::new(mesh_handle, Some(material_name.clone()));

                let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

                let mut transform = base_transform;

                transform.set_translation(Vec3 {
                    x: -GRID_WIDTH / 2.0 + (GRID_WIDTH * alpha_x),
                    y: -GRID_HEIGHT / 2.0 + (GRID_HEIGHT * alpha_y),
                    z: 0.0,
                });

                let entity_node =
                    SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle));

                scene.root.add_child(entity_node)?;
            }
        }
    }

    scene_context.scenes.borrow_mut().push(scene);

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
