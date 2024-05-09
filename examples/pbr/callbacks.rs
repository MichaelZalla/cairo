use cairo::{
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    matrix::Mat4,
    pipeline::Pipeline,
    resource::handle::Handle,
    scene::{
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    time::TimingInfo,
    vec::{vec3::Vec3, vec4::Vec4},
};

pub fn update_scene_graph_node_default(
    framebuffer: &mut Framebuffer,
    shader_context: &mut ShaderContext,
    scene_resources: &mut SceneResources,
    node: &mut SceneNode,
    timing_info: &TimingInfo,
    keyboard_state: &KeyboardState,
    mouse_state: &MouseState,
    game_controller_state: &GameControllerState,
) -> Result<(), String> {
    match node.get_type() {
        SceneNodeType::AmbientLight => {
            shader_context.set_ambient_light(*node.get_handle());

            Ok(())
        }
        SceneNodeType::DirectionalLight => {
            shader_context.set_directional_light(*node.get_handle());

            Ok(())
        }
        SceneNodeType::PointLight => {
            shader_context
                .get_point_lights_mut()
                .push(node.get_handle().unwrap());

            Ok(())
        }
        SceneNodeType::SpotLight => {
            shader_context
                .get_spot_lights_mut()
                .push(node.get_handle().unwrap());

            Ok(())
        }
        SceneNodeType::Camera => match node.get_handle() {
            Some(handle) => match scene_resources.camera.borrow_mut().get_mut(handle) {
                Ok(entry) => {
                    let camera = &mut entry.item;

                    camera.update(
                        timing_info,
                        keyboard_state,
                        mouse_state,
                        game_controller_state,
                    );

                    let camera_view_inverse_transform = camera.get_view_inverse_transform();

                    shader_context
                        .set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

                    shader_context.set_view_inverse_transform(camera_view_inverse_transform);

                    shader_context.set_projection(camera.get_projection());

                    if let Some(lock) = framebuffer.attachments.depth.as_ref() {
                        let mut depth_buffer = lock.borrow_mut();

                        depth_buffer.set_projection_z_near(camera.get_projection_z_near());
                        depth_buffer.set_projection_z_far(camera.get_projection_z_far());
                    }

                    Ok(())
                }
                Err(err) => panic!(
                    "Failed to get Camera from Arena with Handle {:?}: {}",
                    handle, err
                ),
            },
            None => {
                panic!("Encountered a `Camera` node with no resource handle!")
            }
        },
        SceneNodeType::Entity => {
            node.get_transform_mut().set_rotation(Vec3 {
                x: 0.0,
                y: (timing_info.uptime_seconds / 2.0).sin(),
                z: 0.0,
            });

            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn render_scene_graph_node_default(
    pipeline: &mut Pipeline,
    scene_resources: &SceneResources,
    node: &SceneNode,
    current_world_transform: &Mat4,
    skybox_handle: &mut Option<Handle>,
    camera_handle: &mut Option<Handle>,
) -> Result<(), String> {
    let (node_type, handle) = (node.get_type(), node.get_handle());

    match node_type {
        SceneNodeType::Skybox => {
            match handle {
                Some(handle) => {
                    skybox_handle.replace(*handle);
                }
                None => {
                    panic!("Encountered a `Skybox` node with no resource handle!")
                }
            }
            Ok(())
        }
        SceneNodeType::Entity => match handle {
            Some(handle) => {
                let entity_arena = scene_resources.entity.borrow();

                match entity_arena.get(handle) {
                    Ok(entry) => {
                        let entity = &entry.item;

                        pipeline.render_entity(
                            entity,
                            current_world_transform,
                            &scene_resources.mesh.borrow(),
                        );

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
                    camera_handle.replace(*handle);
                }
                None => {
                    panic!("Encountered a `Camera` node with no resource handle!")
                }
            }
            Ok(())
        }
        SceneNodeType::PointLight => match handle {
            Some(handle) => {
                let point_light_arena = scene_resources.point_light.borrow();

                match point_light_arena.get(handle) {
                    Ok(entry) => {
                        let point_light = &entry.item;

                        pipeline.render_point_light(point_light, None, None);

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
                let spot_light_arena = scene_resources.spot_light.borrow();

                match spot_light_arena.get(handle) {
                    Ok(entry) => {
                        let spot_light = &entry.item;

                        // @TODO Migrate light position to node transform.

                        pipeline.render_spot_light(spot_light, None, None);

                        Ok(())
                    }
                    Err(err) => panic!(
                        "Failed to get SpotLight from Arena with Handle {:?}: {}",
                        handle, err
                    ),
                }
            }
            None => {
                panic!("Encountered a `PointLight` node with no resource handle!")
            }
        },
        _ => Ok(()),
    }
}

pub fn render_skybox_node_default(
    pipeline: &mut Pipeline,
    scene_resources: &SceneResources,
    skybox_handle: &Handle,
    camera_handle: &Handle,
) {
    match scene_resources.cubemap_u8.borrow().get(skybox_handle) {
        Ok(entry) => {
            let skybox_cube_map = &entry.item;

            match scene_resources.camera.borrow().get(camera_handle) {
                Ok(entry) => {
                    let skybox_active_camera = &entry.item;

                    pipeline.render_skybox(skybox_cube_map, skybox_active_camera);
                }
                Err(err) => {
                    panic!(
                        "Failed to get Camera from Arena with Handle {:?}: {}",
                        camera_handle, err
                    )
                }
            }
        }
        Err(err) => panic!(
            "Failed to get Entity from Arena with Handle {:?}: {}",
            skybox_handle, err
        ),
    }
}
