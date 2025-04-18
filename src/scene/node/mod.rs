use std::{collections::VecDeque, fmt::Display};

use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::{
    app::App,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    matrix::Mat4,
    resource::handle::Handle,
    serde::PostDeserialize,
    shader::context::ShaderContext,
    transform::Transform3D,
    vec::vec4::Vec4,
};

use super::resources::SceneResources;

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SceneNodeType {
    #[default]
    Scene,
    Environment,
    AmbientLight,
    DirectionalLight,
    Skybox,
    Empty,
    Camera,
    PointLight,
    SpotLight,
    Entity,
}

impl Display for SceneNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SceneNodeType::Scene => "Scene",
                SceneNodeType::Environment => "Environment",
                SceneNodeType::AmbientLight => "Ambient light",
                SceneNodeType::DirectionalLight => "Directional light",
                SceneNodeType::Skybox => "Skybox",
                SceneNodeType::Empty => "Empty",
                SceneNodeType::Camera => "Camera",
                SceneNodeType::PointLight => "Point light",
                SceneNodeType::SpotLight => "Spot light",
                SceneNodeType::Entity => "Entity",
            }
        )
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SceneNodeGlobalTraversalMethod {
    #[default]
    DepthFirst,
    BreadthFirst,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SceneNodeLocalTraversalMethod {
    #[default]
    PreOrder,
    PostOrder,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    uuid: Uuid,
    pub name: Option<String>,
    node_type: SceneNodeType,
    transform: Transform3D,
    handle: Option<Handle>,
    children: Option<Vec<SceneNode>>,
}

impl PostDeserialize for SceneNode {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl SceneNode {
    pub fn new(node_type: SceneNodeType, transform: Transform3D, handle: Option<Handle>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            node_type,
            transform,
            handle,
            ..Default::default()
        }
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn is_type(&self, node_type: SceneNodeType) -> bool {
        self.node_type == node_type
    }

    pub fn is_scene_root(&self) -> bool {
        self.is_type(SceneNodeType::Scene)
    }

    pub fn get_type(&self) -> &SceneNodeType {
        &self.node_type
    }

    pub fn get_transform(&self) -> &Transform3D {
        &self.transform
    }

    pub fn get_transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }

    pub fn get_handle(&self) -> &Option<Handle> {
        &self.handle
    }

    pub fn set_handle(&mut self, handle: Option<Handle>) {
        self.handle = handle;
    }

    pub fn has_children(&self) -> bool {
        match self.children() {
            Some(children) => !children.is_empty(),
            None => false,
        }
    }

    pub fn children(&self) -> &Option<Vec<Self>> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Option<Vec<Self>> {
        &mut self.children
    }

    pub fn add_child(&mut self, node: Self) -> Result<(), String> {
        match node.node_type {
            SceneNodeType::Scene => {
                return Err("Scene node must be the root node.".to_string());
            }
            SceneNodeType::Environment => {
                // Environment node may only be a child of a Scene node.
                if !self.is_scene_root() {
                    return Err("Attempted to add an Environment node as a child to a node that is not a Scene node!".to_string());
                }

                // Only one Environment node may exist per scene at a time.
                if let Some(children) = self.children() {
                    if children
                        .iter()
                        .any(|child| child.is_type(SceneNodeType::Environment))
                    {
                        return Err(
                            "Cannot add multiple Environment nodes to a Scene node!".to_string()
                        );
                    }
                }
            }
            SceneNodeType::AmbientLight
            | SceneNodeType::DirectionalLight
            | SceneNodeType::Skybox => {
                // Node may only be a child of an Environment node.
                if !self.is_type(SceneNodeType::Environment) {
                    return Err(format!("Attempted to add a {} node as a child to a node that is not an Environment node!", node.node_type).to_string());
                }

                // Only one node of this type may exist per scene (environment) at a time.
                if let Some(children) = {
                    let this = &self;
                    &this.children
                } {
                    if children.iter().any(|child| child.is_type(node.node_type)) {
                        return Err(format!(
                            "Cannot add multiple {} nodes to an Environment node!",
                            node.node_type
                        )
                        .to_string());
                    }
                }
            }
            SceneNodeType::Empty => (),
            SceneNodeType::Camera => (),
            SceneNodeType::PointLight => (),
            SceneNodeType::SpotLight => (),
            SceneNodeType::Entity => (),
        }

        match self.children.as_mut() {
            Some(children) => {
                children.push(node);
            }
            None => {
                self.children = Some(vec![node]);
            }
        }

        Ok(())
    }

    pub fn find<P>(&self, predicate: P) -> Result<Option<Handle>, String>
    where
        P: Fn(&SceneNode) -> bool,
    {
        let mut last_matching_node_handle: Option<Handle> = None;

        let mut check_predicate = |_depth: usize,
                                   _current_world_transform: Mat4,
                                   node: &SceneNode|
         -> Result<(), String> {
            if predicate(node) {
                last_matching_node_handle = node.handle;
            }

            Ok(())
        };

        let result = self.visit(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            None,
            &mut check_predicate,
        );

        match result {
            Ok(_) => Ok(last_matching_node_handle),
            Err(err) => Err(err),
        }
    }

    pub fn visit<C>(
        &self,
        global_method: SceneNodeGlobalTraversalMethod,
        local_method: Option<SceneNodeLocalTraversalMethod>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &SceneNode) -> Result<(), String>,
    {
        let local = local_method.unwrap_or_default();

        let current_depth: usize = 0;
        let parent_world_transform = Mat4::identity();

        match global_method {
            SceneNodeGlobalTraversalMethod::DepthFirst => {
                self.visit_dfs(&local, current_depth, parent_world_transform, visit_action)
            }
            SceneNodeGlobalTraversalMethod::BreadthFirst => self.visit_bfs(visit_action),
        }
    }

    pub fn visit_mut<C>(
        &mut self,
        global_method: SceneNodeGlobalTraversalMethod,
        local_method: Option<SceneNodeLocalTraversalMethod>,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &mut SceneNode) -> Result<(), String>,
    {
        let local = local_method.unwrap_or_default();
        let current_depth: usize = 0;
        let parent_world_transform = Mat4::identity();

        match global_method {
            SceneNodeGlobalTraversalMethod::DepthFirst => {
                self.visit_dfs_mut(&local, current_depth, parent_world_transform, visit_action)
            }
            SceneNodeGlobalTraversalMethod::BreadthFirst => self.visit_bfs_mut(visit_action),
        }
    }

    fn visit_dfs<C>(
        &self,
        local_method: &SceneNodeLocalTraversalMethod,
        current_depth: usize,
        parent_world_transform: Mat4,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &SceneNode) -> Result<(), String>,
    {
        let current_world_transform = *(self.transform.mat()) * parent_world_transform;

        match local_method {
            SceneNodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, current_world_transform, self)?;

                if let Some(children) = &self.children {
                    for child in children {
                        child.visit_dfs(
                            local_method,
                            current_depth + 1,
                            current_world_transform,
                            visit_action,
                        )?;
                    }
                }

                Ok(())
            }
            SceneNodeLocalTraversalMethod::PostOrder => {
                if let Some(children) = &self.children {
                    for child in children {
                        child.visit_dfs(
                            local_method,
                            current_depth + 1,
                            current_world_transform,
                            visit_action,
                        )?;
                    }
                }

                visit_action(current_depth, current_world_transform, self)
            }
        }
    }

    fn visit_dfs_mut<C>(
        &mut self,
        local_method: &SceneNodeLocalTraversalMethod,
        current_depth: usize,
        parent_world_transform: Mat4,
        visit_action: &mut C,
    ) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &mut Self) -> Result<(), String>,
    {
        let current_world_transform = *(self.transform.mat()) * parent_world_transform;

        match local_method {
            SceneNodeLocalTraversalMethod::PreOrder => {
                visit_action(current_depth, current_world_transform, self)?;

                if let Some(children) = self.children.as_mut() {
                    for child in children {
                        child.visit_dfs_mut(
                            local_method,
                            current_depth + 1,
                            current_world_transform,
                            visit_action,
                        )?;
                    }
                }

                Ok(())
            }
            SceneNodeLocalTraversalMethod::PostOrder => {
                if let Some(children) = self.children.as_mut() {
                    for child in children {
                        child.visit_dfs_mut(
                            local_method,
                            current_depth + 1,
                            current_world_transform,
                            visit_action,
                        )?;
                    }
                }

                visit_action(current_depth, current_world_transform, self)
            }
        }
    }

    fn visit_bfs<C>(&self, visit_action: &mut C) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &SceneNode) -> Result<(), String>,
    {
        let mut frontier: VecDeque<(usize, Mat4, &SceneNode)> = VecDeque::new();

        let current_depth: usize = 0;
        let parent_world_transform = *self.transform.mat();

        frontier.push_front((current_depth, parent_world_transform, self));

        while !frontier.is_empty() {
            let (current_depth, parent_world_transform, current_node) =
                frontier.pop_front().unwrap();

            let current_world_transform = *(current_node.transform.mat()) * parent_world_transform;

            visit_action(current_depth, current_world_transform, current_node)?;

            if let Some(children) = &current_node.children {
                for child in children {
                    frontier.push_back((current_depth + 1, current_world_transform, child));
                }
            }
        }

        Ok(())
    }

    fn visit_bfs_mut<C>(&mut self, visit_action: &mut C) -> Result<(), String>
    where
        C: FnMut(usize, Mat4, &mut Self) -> Result<(), String>,
    {
        let mut frontier: VecDeque<(usize, Mat4, &mut Self)> = VecDeque::new();

        let current_depth: usize = 0;
        let parent_world_transform = *self.transform.mat();

        frontier.push_front((current_depth, parent_world_transform, self));

        while !frontier.is_empty() {
            let (current_depth, parent_world_transform, current_node) =
                frontier.pop_front().unwrap();

            let current_world_transform = *(current_node.transform.mat()) * parent_world_transform;

            visit_action(current_depth, current_world_transform, current_node)?;

            if let Some(children) = current_node.children.as_mut() {
                for child in children {
                    frontier.push_back((current_depth + 1, current_world_transform, child));
                }
            }
        }

        Ok(())
    }

    pub fn update(
        &mut self,
        current_world_transform: &Mat4,
        resources: &SceneResources,
        app: &App,
        mouse_state: &MouseState,
        keyboard_state: &KeyboardState,
        game_controller_state: &GameControllerState,
        shader_context: &mut ShaderContext,
    ) -> Result<(), String> {
        let (node_type, handle) = (self.get_type(), self.get_handle());

        match node_type {
            SceneNodeType::Camera => match handle {
                Some(handle) => {
                    let mut camera_arena = resources.camera.borrow_mut();

                    match camera_arena.get_mut(handle) {
                        Ok(entry) => {
                            let camera = &mut entry.item;

                            if !camera.is_active {
                                return Ok(());
                            }

                            camera.update(
                                &app.timing_info,
                                keyboard_state,
                                mouse_state,
                                game_controller_state,
                            );

                            camera.recompute_world_space_frustum();

                            camera.update_shader_context(shader_context);

                            Ok(())
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
            },
            SceneNodeType::Skybox => match handle {
                Some(handle) => {
                    let mut skybox_arena = resources.skybox.borrow_mut();

                    match skybox_arena.get_mut(handle) {
                        Ok(entry) => {
                            let skybox = &mut entry.item;

                            shader_context.set_ambient_radiance_map(skybox.radiance);

                            shader_context.set_ambient_diffuse_irradiance_map(skybox.irradiance);

                            shader_context.set_ambient_specular_prefiltered_environment_map(
                                skybox.specular_prefiltered_environment,
                            );

                            shader_context.set_ambient_specular_brdf_integration_map(
                                skybox.ambient_specular_brdf_integration,
                            );

                            shader_context.set_skybox_transform(Some(*current_world_transform));

                            Ok(())
                        }
                        Err(err) => panic!(
                            "Failed to get Skybox from Arena with Handle {:?}: {}",
                            handle, err
                        ),
                    }
                }
                None => {
                    panic!("Encountered a `Skybox` node with no resource handle!")
                }
            },
            SceneNodeType::AmbientLight => match handle {
                Some(handle) => {
                    shader_context.set_ambient_light(Some(*handle));

                    Ok(())
                }
                None => {
                    panic!("Encountered a `AmbientLight` node with no resource handle!")
                }
            },
            SceneNodeType::DirectionalLight => match handle {
                Some(handle) => {
                    shader_context.set_directional_light(Some(*handle));

                    let mut directional_light_arena = resources.directional_light.borrow_mut();

                    if let Ok(entry) = directional_light_arena.get_mut(handle) {
                        let directional_light = &mut entry.item;

                        if let (Some(_), Some(_)) = (
                            &directional_light.shadow_maps,
                            &directional_light.shadow_map_rendering_context,
                        ) {
                            let camera_arena = resources.camera.borrow();

                            if let Some(view_camera) = camera_arena
                                .entries
                                .iter()
                                .flatten()
                                .find(|entry| entry.item.is_active)
                                .map(|entry| &entry.item)
                            {
                                directional_light.update_shadow_map_cameras(view_camera);

                                if let Some(shadow_map_cameras) =
                                    &directional_light.shadow_map_cameras
                                {
                                    let transforms: Vec<(f32, Mat4)> = shadow_map_cameras
                                        .iter()
                                        .map(|(far_z, camera)| {
                                            (
                                                *far_z,
                                                camera.get_view_inverse_transform()
                                                    * camera.get_projection(),
                                            )
                                        })
                                        .collect();

                                    shader_context
                                        .set_directional_light_view_projections(Some(transforms));
                                }
                            }
                        }
                    }

                    Ok(())
                }
                None => {
                    panic!("Encountered a `DirectionalLight` node with no resource handle!")
                }
            },
            SceneNodeType::PointLight => match handle {
                Some(handle) => {
                    let mut point_light_arena = resources.point_light.borrow_mut();

                    match point_light_arena.get_mut(handle) {
                        Ok(entry) => {
                            let point_light = &mut entry.item;

                            point_light.position = (Vec4::new(Default::default(), 1.0)
                                * (*current_world_transform))
                                .to_vec3();

                            if point_light.shadow_map_cameras.is_some() {
                                point_light.update_shadow_map_cameras();
                            }

                            shader_context.get_point_lights_mut().push(*handle);

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
                    let mut spot_light_arena = resources.spot_light.borrow_mut();

                    match spot_light_arena.get_mut(handle) {
                        Ok(entry) => {
                            let spot_light = &mut entry.item;

                            spot_light.look_vector.set_position(
                                (Vec4::new(Default::default(), 1.0) * (*current_world_transform))
                                    .to_vec3(),
                            );

                            if spot_light.shadow_map_camera.is_some() {
                                spot_light.update_shadow_map_camera();
                            }

                            shader_context.get_spot_lights_mut().push(*handle);

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
            _ => Ok(()),
        }
    }
}

impl Display for SceneNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uuid_suffix = match &self.handle {
            Some(handle) => format!(" | {}", handle.uuid),
            None => "".to_string(),
        };

        let children_suffix = match &self.children {
            Some(children) => {
                if children.len() > 1 {
                    format!(" ({} children)", children.len())
                } else {
                    " (1 child)".to_string()
                }
            }
            None => "".to_string(),
        };

        write!(f, "{}{}{}", self.node_type, uuid_suffix, children_suffix)?;

        Ok(())
    }
}
