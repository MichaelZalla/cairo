use std::{
    cell::RefCell,
    fmt::{Display, Error},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    render::{culling::FaceCullingReject, options::RenderPassFlag, Renderer},
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

use options::SceneGraphRenderOptions;

pub mod options;

type UpdateSceneGraphNodeCallback = dyn Fn(
    &Mat4,
    &mut SceneNode,
    &SceneResources,
    &App,
    &MouseState,
    &KeyboardState,
    &GameControllerState,
    &mut ShaderContext,
) -> Result<bool, String>;

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

    pub fn update(
        &mut self,
        resources: &SceneResources,
        shader_context: &mut ShaderContext,
        app: &App,
        mouse_state: &MouseState,
        keyboard_state: &KeyboardState,
        game_controller_state: &GameControllerState,
        mut update_node: Option<Rc<UpdateSceneGraphNodeCallback>>,
    ) -> Result<(), String> {
        shader_context.clear_lights();

        self.root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut |_current_depth: usize, current_world_transform: Mat4, node: &mut SceneNode| {
                let mut was_handled = false;

                if let Some(callback) = update_node.as_mut() {
                    match (*callback)(
                        &current_world_transform,
                        node,
                        resources,
                        app,
                        mouse_state,
                        keyboard_state,
                        game_controller_state,
                        shader_context,
                    ) {
                        Ok(result) => was_handled = result,
                        Err(e) => return Err(e),
                    }
                };

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
            },
        )?;

        Ok(())
    }

    pub fn render(
        &self,
        resources: &SceneResources,
        renderer_rc: &RefCell<dyn Renderer>,
        render_options: Option<SceneGraphRenderOptions>,
    ) -> Result<(), String> {
        let options = render_options.unwrap_or_default();

        // Begin frame

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();
        }

        // Render the scene.

        // 1. Collect handles to active camera, clipping camera, active skybox, etc.
        // 2. Render shadow maps for directional light and point lights.
        // 3. Render opaque entities into the depth and deferred HDR buffers.
        // 4. Render semi-transparent entities into the accumulation and revealage buffers.

        let active_camera_handle_rc = RefCell::new(options.camera);
        let clipping_camera_handle_rc = RefCell::new(options.camera);

        let active_skybox_handle_rc: RefCell<Option<Handle>> = Default::default();
        let active_skybox_transform_rc: RefCell<Option<Mat4>> = Default::default();

        let mut collect_handles = |_current_depth: usize,
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

                                let mut active_camera_handle = active_camera_handle_rc.borrow_mut();

                                if camera.is_active && active_camera_handle.is_none() {
                                    active_camera_handle.replace(*handle);
                                }

                                let mut clipping_camera_handle =
                                    clipping_camera_handle_rc.borrow_mut();

                                if clipping_camera_handle.is_none() {
                                    clipping_camera_handle.replace(*handle);
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
                        panic!("Encountered a `Camera` node with no handle!")
                    }
                },
                SceneNodeType::Skybox => {
                    match handle {
                        Some(handle) => {
                            let mut active_skybox_handle = active_skybox_handle_rc.borrow_mut();

                            active_skybox_handle.replace(*handle);

                            let mut active_skybox_transform =
                                active_skybox_transform_rc.borrow_mut();

                            active_skybox_transform.replace(current_world_transform);
                        }
                        None => {
                            panic!("Encountered a `Skybox` node with no handle!")
                        }
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        };

        let mut render_lights = |_current_depth: usize,
                                 current_world_transform: Mat4,
                                 node: &SceneNode|
         -> Result<(), String> {
            let mut renderer = renderer_rc.borrow_mut();

            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::AmbientLight => {
                    match handle {
                        Some(ambient_light_handle) => {
                            let ambient_light_arena = resources.ambient_light.borrow();

                            match ambient_light_arena.get(ambient_light_handle) {
                                Ok(entry) => {
                                    let ambient_light = &entry.item;

                                    renderer.render_ambient_light(
                                        &current_world_transform,
                                        ambient_light,
                                    );
                                }
                                Err(err) => panic!(
                                    "Failed to get AmbientLight from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered an `AmbientLight` node with no handle!")
                        }
                    }

                    Ok(())
                }
                SceneNodeType::DirectionalLight => {
                    match handle {
                        Some(directional_light_handle) => {
                            let mut directional_light_arena =
                                resources.directional_light.borrow_mut();

                            match directional_light_arena.get_mut(directional_light_handle) {
                                Ok(entry) => {
                                    let directional_light = &mut entry.item;

                                    renderer.render_directional_light(&current_world_transform, directional_light);
                                }
                                Err(err) => panic!(
                                    "Failed to get DirectionalLight from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered a `DirectionalLight` node with no handle!")
                        }
                    }

                    Ok(())
                }
                SceneNodeType::PointLight => match handle {
                    Some(point_light_handle) => {
                        let mut point_light_arena = resources.point_light.borrow_mut();

                        match point_light_arena.get_mut(point_light_handle) {
                            Ok(entry) => {
                                let point_light = &mut entry.item;

                                renderer.render_point_light(&current_world_transform, point_light);

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get PointLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `PointLight` node with no handle!")
                    }
                },
                SceneNodeType::SpotLight => match handle {
                    Some(spot_light_handle) => {
                        let spot_light_arena = resources.spot_light.borrow();

                        match spot_light_arena.get(spot_light_handle) {
                            Ok(entry) => {
                                let spot_light = &entry.item;

                                renderer.render_spot_light(&current_world_transform, spot_light);

                                Ok(())
                            }
                            Err(err) => panic!(
                                "Failed to get PointLight from Arena with Handle {:?}: {}",
                                handle, err
                            ),
                        }
                    }
                    None => {
                        panic!("Encountered a `PointLight` node with no handle!")
                    }
                },
                _ => Ok(()),
            }
        };

        let mut render_cameras = |_current_depth: usize,
                                  _current_world_transform: Mat4,
                                  node: &SceneNode|
         -> Result<(), String> {
            let mut renderer = renderer_rc.borrow_mut();

            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Camera => match handle {
                    Some(handle) => {
                        let camera_arena = resources.camera.borrow();

                        match camera_arena.get(handle) {
                            Ok(entry) => {
                                let camera = &entry.item;

                                if !camera.is_active && options.draw_cameras {
                                    renderer.render_camera(camera, Some(color::ORANGE));
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
                        panic!("Encountered a `Camera` node with no handle!")
                    }
                },
                SceneNodeType::DirectionalLight => {
                    match handle {
                        Some(directional_light_handle) => {
                            let mut directional_light_arena =
                                resources.directional_light.borrow_mut();

                            match directional_light_arena.get_mut(directional_light_handle) {
                                Ok(entry) => {
                                    let directional_light = &mut entry.item;

                                    if options.draw_shadow_map_cameras {
                                        if let Some(shadow_map_cameras) = directional_light.shadow_map_cameras.as_ref() {
                                            for (index, (_far_z, camera)) in shadow_map_cameras.iter().enumerate() {
                                                let frustum_color = [
                                                    color::RED,
                                                    color::GREEN,
                                                    color::BLUE,
                                                ][index];
    
                                                renderer.render_camera(camera, Some(frustum_color));
                                            }
                                        }
                                    }
                                }
                                Err(err) => panic!(
                                    "Failed to get DirectionalLight from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered a `DirectionalLight` node with no handle!")
                        }
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        };

        let mut render_shadow_maps = |_current_depth: usize,
                                      _current_world_transform: Mat4,
                                      node: &SceneNode|
         -> Result<(), String> {
            let renderer = renderer_rc.borrow();

            let render_pass_flags = renderer.get_options().render_pass_flags;

            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::DirectionalLight => {
                    match handle {
                        Some(directional_light_handle) => {
                            let mut directional_light_arena =
                                resources.directional_light.borrow_mut();

                            match directional_light_arena.get_mut(directional_light_handle) {
                                Ok(entry) => {
                                    let directional_light = &mut entry.item;

                                    if let (Some(_), Some(_), true) = (
                                        directional_light.shadow_maps.as_ref(),
                                        directional_light.shadow_map_rendering_context.as_ref(),
                                        render_pass_flags.contains(RenderPassFlag::Lighting))
                                    {
                                        directional_light.update_shadow_maps(resources, self)?;
                                    }
                                }
                                Err(err) => panic!(
                                    "Failed to get DirectionalLight from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered a `DirectionalLight` node with no handle!")
                        }
                    }

                    Ok(())
                }
                SceneNodeType::PointLight => {
                    if options.is_shadow_map_render {
                        return Ok(());
                    }

                    match handle {
                        Some(point_light_handle) => {
                            let mut point_light_arena = resources.point_light.borrow_mut();

                            match point_light_arena.get_mut(point_light_handle) {
                                Ok(entry) => {
                                    let point_light = &mut entry.item;

                                    if let (Some(_), Some(_), true) = (
                                        point_light.shadow_map.as_ref(),
                                        point_light.shadow_map_rendering_context.as_ref(),
                                        render_pass_flags.contains(RenderPassFlag::Lighting),
                                    ) {
                                        point_light.update_shadow_map(resources, self)?;
                                    }

                                    Ok(())
                                }
                                Err(err) => panic!(
                                    "Failed to get PointLight from Arena with Handle {:?}: {}",
                                    handle, err
                                ),
                            }
                        }
                        None => {
                            panic!("Encountered a `PointLight` node with no handle!")
                        }
                    }
                }
                _ => Ok(()),
            }
        };

        let mut render_opaque_entities = |_current_depth: usize,
                                          current_world_transform: Mat4,
                                          node: &SceneNode|
         -> Result<(), String> {
            let mut renderer = renderer_rc.borrow_mut();

            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let entity_arena = resources.entity.borrow();

                        match entity_arena.get(handle) {
                            Ok(entry) => {
                                let entity = &entry.item;

                                if let Some(material_handle) = entity.material.as_ref() {
                                    let material_arena = resources.material.borrow();

                                    if let Ok(entry) = material_arena.get(material_handle) {
                                        let material = &entry.item;

                                        if material.transparency > 0.0 {
                                            // Object is semi-transparent.

                                            return Ok(());
                                        }
                                    }
                                }

                                let mesh_arena = resources.mesh.borrow();

                                match mesh_arena.get(&entity.mesh) {
                                    Ok(entry) => {
                                        let entity_mesh = &entry.item;

                                        let clipping_camera_handle =
                                            clipping_camera_handle_rc.borrow();

                                        let clipping_camera_frustum = match clipping_camera_handle
                                            .as_ref()
                                        {
                                            Some(camera_handle) => {
                                                let camera_arena = resources.camera.borrow();

                                                match camera_arena.get(camera_handle) {
                                                    Ok(entry) => Some(*entry.item.get_frustum()),
                                                    Err(err) => panic!(
                                                        "Failed to get Camera from Arena with Handle {:?}: {}",
                                                        entity.mesh, err
                                                    ),
                                                }
                                            }
                                            None => None,
                                        };

                                        let _was_drawn = renderer.render_entity(
                                            &current_world_transform,
                                            &clipping_camera_frustum,
                                            entity_mesh,
                                            &entity.material,
                                        );

                                        if let Some(bvh) = entity_mesh.static_triangle_bvh.as_ref()
                                        {
                                            // Render the BVH root's AABB.

                                            let root = &bvh.root;

                                            renderer.render_aabb(
                                                &root.aabb,
                                                &current_world_transform,
                                                color::GREEN,
                                            );
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
                        panic!("Encountered a `Entity` node with no handle!")
                    }
                },
                _ => Ok(()),
            }
        };

        let mut render_semi_transparent_entities = |_current_depth: usize,
                                                    current_world_transform: Mat4,
                                                    node: &SceneNode|
         -> Result<(), String> {
            let mut renderer = renderer_rc.borrow_mut();

            let (node_type, handle) = (node.get_type(), node.get_handle());

            match node_type {
                SceneNodeType::Entity => match handle {
                    Some(handle) => {
                        let entity_arena = resources.entity.borrow();

                        match entity_arena.get(handle) {
                            Ok(entry) => {
                                let entity = &entry.item;

                                match entity.material.as_ref() {
                                    Some(material_handle) => {
                                        let material_arena = resources.material.borrow();

                                        if let Ok(entry) = material_arena.get(material_handle) {
                                            let material = &entry.item;

                                            if material.transparency == 0.0 {
                                                // Object is opaque.

                                                return Ok(());
                                            }
                                        }
                                    }
                                    None => {
                                        return Ok(());
                                    }
                                }

                                let mesh_arena = resources.mesh.borrow();

                                match mesh_arena.get(&entity.mesh) {
                                    Ok(entry) => {
                                        let entity_mesh = &entry.item;

                                        let clipping_camera_handle =
                                            clipping_camera_handle_rc.borrow();

                                        let clipping_camera_frustum = match clipping_camera_handle
                                            .as_ref()
                                        {
                                            Some(camera_handle) => {
                                                let camera_arena = resources.camera.borrow();

                                                match camera_arena.get(camera_handle) {
                                                    Ok(entry) => Some(*entry.item.get_frustum()),
                                                    Err(err) => panic!(
                                                        "Failed to get Camera from Arena with Handle {:?}: {}",
                                                        entity.mesh, err
                                                    ),
                                                }
                                            }
                                            None => None,
                                        };

                                        let _was_drawn = renderer.render_entity(
                                            &current_world_transform,
                                            &clipping_camera_frustum,
                                            entity_mesh,
                                            &entity.material,
                                        );

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
                        panic!("Encountered a `Entity` node with no handle!")
                    }
                },
                _ => Ok(()),
            }
        };

        // Collect handles.

        self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut collect_handles,
        )?;

        // Render shadow maps.

        if !options.is_shadow_map_render {
            self.root.visit(
                SceneNodeGlobalTraversalMethod::DepthFirst,
                Some(SceneNodeLocalTraversalMethod::PostOrder),
                &mut render_shadow_maps,
            )?;
        }

        // Render opaque entities.

        self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut render_opaque_entities,
        )?;

        // Render semi-transparent entities.

        let original_face_culling_reject;

        {
            let mut renderer = renderer_rc.borrow_mut();

            let options = renderer.get_options_mut();

            original_face_culling_reject = options.rasterizer_options.face_culling_strategy.reject;

            options.rasterizer_options.face_culling_strategy.reject = FaceCullingReject::None;
        }

        self.root.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            Some(SceneNodeLocalTraversalMethod::PostOrder),
            &mut render_semi_transparent_entities,
        )?;

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer
                .get_options_mut()
                .rasterizer_options
                .face_culling_strategy
                .reject = original_face_culling_reject;
        }

        if !options.is_shadow_map_render {
            // Draw lights.

            if options.draw_lights {
                self.root.visit(
                    SceneNodeGlobalTraversalMethod::DepthFirst,
                    Some(SceneNodeLocalTraversalMethod::PostOrder),
                    &mut render_lights,
                )?;
            }

            // Draw cameras.

            if options.draw_cameras {
                self.root.visit(
                    SceneNodeGlobalTraversalMethod::DepthFirst,
                    Some(SceneNodeLocalTraversalMethod::PostOrder),
                    &mut render_cameras,
                )?;
            }
        }

        // Render skybox

        {
            let active_camera_handle = active_camera_handle_rc.borrow();
            let active_skybox_handle = active_skybox_handle_rc.borrow();
            let active_skybox_transform = active_skybox_transform_rc.borrow();

            if let (Some(camera_handle), Some(skybox_handle), Some(skybox_transform)) = (
                active_camera_handle.as_ref(),
                active_skybox_handle.as_ref(),
                active_skybox_transform.as_ref(),
            ) {
                if let (Ok(camera_entry), Ok(skybox_entry)) = (
                    resources.camera.borrow().get(camera_handle),
                    resources.skybox.borrow().get(skybox_handle),
                ) {
                    let camera = &camera_entry.item;
                    let skybox = &skybox_entry.item;

                    if let Some(cubemap_handle) = skybox.radiance {
                        let mut renderer = renderer_rc.borrow_mut();

                        if skybox.is_hdr {
                            match resources.cubemap_vec3.borrow().get(&cubemap_handle) {
                                Ok(entry) => {
                                    let cubemap = &entry.item;

                                    renderer.render_skybox_hdr(
                                        cubemap,
                                        camera,
                                        Some(*skybox_transform),
                                    );
                                }
                                Err(e) => panic!("{}", e),
                            }
                        } else {
                            match resources.cubemap_u8.borrow().get(&cubemap_handle) {
                                Ok(entry) => {
                                    let cubemap = &entry.item;

                                    renderer.render_skybox(
                                        cubemap,
                                        camera,
                                        Some(*skybox_transform),
                                    );
                                }
                                Err(e) => panic!("{}", e),
                            }
                        }
                    }
                }
            }
        }

        // End frame

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.end_frame();
        }

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
