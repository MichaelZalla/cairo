use std::rc::Rc;

use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh::Mesh,
    resource::{arena::Arena, handle::Handle},
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{ambient_light::AmbientLight, directional_light::DirectionalLight},
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    texture::map::TextureMap,
};

#[allow(clippy::too_many_arguments)]
pub fn make_collision_physics_scene(
    _resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    texture_u8_arena: &mut Arena<TextureMap>,
    rendering_context: &ApplicationRenderingContext,
    level_meshes: Vec<Mesh>,
    level_mesh_handle: &mut Handle,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    for entry in material_arena.entries.iter_mut().flatten() {
        let material = &mut entry.item;

        material.roughness = 1.0;
        material.metallic = 0.0;

        material.load_all_maps(texture_u8_arena, rendering_context)?;
    }

    // Assign the level mesh to an entity.

    for mesh in &level_meshes {
        let level_entity_node = {
            let material_handle = mesh.material;

            let mesh_handle = mesh_arena.insert(mesh.to_owned());

            *level_mesh_handle = mesh_handle;

            let entity_handle = entity_arena.insert(Entity::new(mesh_handle, material_handle));

            SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            )
        };

        scene.root.add_child(level_entity_node)?;
    }

    Ok((scene, shader_context))
}
