use std::{cell::RefCell, fmt::{Display, Error}};

use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    matrix::Mat4,
    render::Renderer,
    resource::handle::Handle,
    serde::PostDeserialize,
    shader::context::ShaderContext,
};

use super::{
    node::{
        SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeLocalTraversalMethod, SceneNodeType,
    },
    resources::SceneResources,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SceneGraphRenderOptions {
    pub draw_lights: bool,
    pub draw_cameras: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SceneGraph {
    pub root: SceneNode,
}

impl PostDeserialize for SceneGraph {
    fn post_deserialize(&mut self) {
        self.root.post_deserialize();
    }
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            root: SceneNode::new(SceneNodeType::Scene, Default::default(), None),
        }
    }

    pub fn update<C>(
        &mut self,
        resources: &SceneResources,
        shader_context: &mut ShaderContext,
        app: &App,
        mouse_state: &MouseState,
        keyboard_state: &KeyboardState,
        game_controller_state: &GameControllerState,
        update_node: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(
            &Mat4,
            &mut SceneNode,
            &SceneResources,
            &App,
            &MouseState,
            &KeyboardState,
            &GameControllerState,
            &mut ShaderContext,
        ) -> Result<bool, String>,
    {
        self.root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut |_current_depth: usize, current_world_transform: Mat4, node: &mut SceneNode| {
                match update_node(
                    &current_world_transform,
                    node,
                    resources,
                    app,
                    mouse_state,
                    keyboard_state,
                    game_controller_state,
                    shader_context,
                ) {
                    Ok(was_handled) => {
                        if !was_handled {
                            return node.update(
                                &current_world_transform,
                                resources,
                                app,
                                mouse_state,
                                keyboard_state,
                                game_controller_state,
                                shader_context,
                            );
                        }

                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            },
        )?;

        Ok(())
    }

    pub fn render(
        &self,
        resources: &SceneResources,
        renderer_rc: &RefCell<dyn Renderer>,
        options: Option<SceneGraphRenderOptions>,
    ) -> Result<(), String> {
        let mut renderer = renderer_rc.borrow_mut();

        // Begin frame

        renderer.begin_frame();

        // Render scene.

        let mut active_camera_handle: Option<Handle> = None;
        let mut clipping_camera_handle: Option<Handle> = None;
        let mut active_skybox_handle: Option<Handle> = None;
        let mut active_skybox_transform: Option<Mat4> = None;

        let mut entities_total: u32 = 0;
        let mut entities_drawn: u32 = 0;
        let mut entities_culled: u32 = 0;

        let mut render_scene_graph_node = |_current_depth: usize,
                                           current_world_transform: Mat4,
                                           node: &SceneNode|
         -> Result<(), String> {
            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Camera => match handle {
                    Some(handle) => {
                        let camera_arena = resources.camera.borrow();

                        match camera_arena.get(handle) {
                            Ok(entry) => {
                                let camera = &entry.item;

                                if camera.is_active {
                                    active_camera_handle.replace(*handle);
                                    clipping_camera_handle.replace(*handle);
                                } else if let Some(options) = &options {
                                    if options.draw_cameras {
                                        renderer.render_camera(camera, Some(&color::YELLOW));
                                    }
                                }
                            }
                            Err(err) => panic!(
                                "Failed to get Camera from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }

                        Ok(())
                    }
                    None => {
                        panic!("Encountered a `Camera` node with no resource handle!")
                    }
                },
                SceneNodeType::Skybox => {
                    match handle {
                        Some(handle) => {
                            active_skybox_handle.replace(*handle);
                            active_skybox_transform.replace(current_world_transform);
                        }
                        None => {
                            panic!("Encountered a `Skybox` node with no resource handle!")
                        }
                    }

                    Ok(())
                }
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let mesh_arena = resources.mesh.borrow();
                        let entity_arena = resources.entity.borrow();
                        let camera_arena = resources.camera.borrow();

                        match entity_arena.get(handle) {
                            Ok(entry) => {
                                let entity = &entry.item;

                                match mesh_arena.get(&entity.mesh) {
                                    Ok(entry) => {
                                        let entity_mesh = &entry.item;

                                        let clipping_camera_frustum = match clipping_camera_handle {
                                            Some(camera_handle) => {
                                                match camera_arena.get(&camera_handle) {
                                                    Ok(entry) => Some(*entry.item.get_frustum()),
                                                    Err(err) => panic!(
                                                        "Failed to get Camera from Arena with Handle {:?}: {}",
                                                        entity.mesh, err
                                                    ),
                                                }
                                            }
                                            None => None,
                                        };

                                        let was_drawn = renderer.render_entity(
                                            &current_world_transform,
                                            &clipping_camera_frustum,
                                            entity_mesh,
                                            &entity.material,
                                        );

                                        entities_total += 1;

                                        if was_drawn {
                                            entities_drawn += 1
                                        } else {
                                            entities_culled += 1
                                        }

                                        Ok(())
                                    }
                                    Err(err) => panic!(
                                        "Failed to get Mesh from Arena with Handle {:?}: {}",
                                        entity.mesh, err
                                    ),
                                }
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
                SceneNodeType::PointLight => {
                    if let Some(options) = &options {
                        if !options.draw_lights {
                            return Ok(());
                        }
                    }

                    match handle {
                        Some(point_light_handle) => {
                            let camera_arena = resources.camera.borrow();
                            let point_light_arena = resources.point_light.borrow();

                            match point_light_arena.get(point_light_handle) {
                                Ok(entry) => {
                                    let point_light = &entry.item;

                                    match active_camera_handle {
                                        Some(camera_handle) => {
                                            match camera_arena.get(&camera_handle) {
                                                Ok(entry) => {
                                                    let active_camera = &entry.item;
            
                                                    renderer.render_point_light(
                                                        point_light,
                                                        Some(active_camera),
                                                        Some(&mut resources.material.borrow_mut()),
                                                    );
            
                                                    Ok(())
                                                }
                                                Err(err) => panic!(
                                                    "Failed to get Camera from Arena with Handle {:?}: {}",
                                                    handle, err
                                                ),
                                            }
                                        }
                                        None => Ok(()),
                                    }
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
                    }
                }
                SceneNodeType::SpotLight => {
                    if let Some(options) = &options {
                        if !options.draw_lights {
                            return Ok(());
                        }
                    }

                    match handle {
                        Some(spot_light_handle) => {
                            let camera_arena = resources.camera.borrow();
                            let mut materials = resources.material.borrow_mut();
                            let spot_light_arena = resources.spot_light.borrow();

                            match active_camera_handle {
                                Some(camera_handle) => match camera_arena.get(&camera_handle) {
                                    Ok(entry) => {
                                        let active_camera = &entry.item;

                                        match spot_light_arena.get(spot_light_handle) {
                                                Ok(entry) => {
                                                    let spot_light = &entry.item;
            
                                                    renderer.render_spot_light(
                                                        spot_light,
                                                        Some(active_camera),
                                                        Some(&mut materials),
                                                    );
            
                                                    Ok(())
                                                }
                                                Err(err) => panic!(
                                                    "Failed to get PointLight from Arena with Handle {:?}: {}",
                                                    handle, err
                                                ),
                                            }
                                    }
                                    Err(err) => panic!(
                                        "Failed to get Camera from Arena with Handle {:?}: {}",
                                        handle, err
                                    ),
                                },
                                None => Ok(()),
                            }
                        }
                        None => {
                            panic!("Encountered a `PointLight` node with no resource handle!")
                        }
                    }
                }
                _ => Ok(()),
            }
        };

        // Traverse the scene graph and render its nodes.

        self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut render_scene_graph_node,
        )?;

        // End frame

        if let (Some(camera_handle), Some(skybox_handle), Some(skybox_transform)) =
            (active_camera_handle, active_skybox_handle, active_skybox_transform)
        {
            if let (Ok(camera_entry), Ok(skybox_entry)) = (
                resources.camera.borrow().get(&camera_handle),
                resources.skybox.borrow().get(&skybox_handle),
            ) {
                let camera = &camera_entry.item;
                let skybox = &skybox_entry.item;

                if let Some(cubemap_handle) = skybox.radiance {
                    if skybox.is_hdr {
                        match resources.cubemap_vec3.borrow().get(&cubemap_handle) {
                            Ok(entry) => {
                                let cubemap = &entry.item;

                                renderer.render_skybox_hdr(cubemap, camera, Some(skybox_transform));
                            }
                            Err(e) => panic!("{}", e),
                        }
                    } else {
                        match resources.cubemap_u8.borrow().get(&cubemap_handle) {
                            Ok(entry) => {
                                let cubemap = &entry.item;

                                renderer.render_skybox(cubemap, camera, Some(skybox_transform));
                            }
                            Err(e) => panic!("{}", e),
                        }
                    }
                }
            }
        }

        renderer.end_frame();

        Ok(())
    }
}

impl Display for SceneGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_node_to_formatter = |current_depth: usize,
                                           _world_transform: Mat4,
                                           node: &SceneNode|
         -> Result<(), String> {
            match write!(
                f,
                "{}{}",
                "   ".repeat((current_depth as i8 - 1).max(0) as usize),
                if current_depth > 0 { "|- " } else { "" }
            ) {
                Ok(()) => (),
                Err(err) => return Err(err.to_string()),
            }

            match writeln!(f, "{}", node) {
                Ok(()) => Ok(()),
                Err(err) => Err(err.to_string()),
            }
        };

        match self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PreOrder),
            &mut write_node_to_formatter,
        ) {
            Ok(()) => Ok(()),
            Err(_err) => Err(Error),
        }
    }
}
