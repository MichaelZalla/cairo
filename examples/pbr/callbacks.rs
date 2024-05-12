use cairo::{
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    scene::{
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    time::TimingInfo,
    vec::vec4::Vec4,
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
        // SceneNodeType::Entity => {
        //     node.get_transform_mut().set_rotation(Vec3 {
        //         x: 0.0,
        //         y: (timing_info.uptime_seconds / 2.0).sin(),
        //         z: 0.0,
        //     });

        //     Ok(())
        // }
        _ => Ok(()),
    }
}
