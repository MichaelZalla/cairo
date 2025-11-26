use std::{f32::consts::PI, rc::Rc};

use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::{
        Mesh,
        obj::load::{LoadObjResult, ProcessGeometryFlags, load_obj},
    },
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{ambient_light::AmbientLight, directional_light::DirectionalLight},
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    texture::map::TextureMap,
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

#[allow(clippy::too_many_arguments)]
pub fn make_ssao_scene(
    resources: &Rc<SceneResources>,
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
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    scene.root.visit_mut(
        SceneNodeGlobalTraversalMethod::DepthFirst,
        None,
        &mut |_current_depth: usize, _current_world_transform: Mat4, node: &mut SceneNode| {
            match node.get_type() {
                SceneNodeType::AmbientLight => {
                    if let Some(handle) = node.get_handle()
                        && let Ok(entry) = ambient_light_arena.get_mut(handle)
                    {
                        let ambient_light = &mut entry.item;

                        ambient_light.intensities = Vec3 {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
                        } * 0.3;
                    }

                    Ok(())
                }
                SceneNodeType::DirectionalLight => {
                    let transform = node.get_transform_mut();

                    transform.set_translation(Vec3 {
                        x: 0.0,
                        y: 15.0,
                        z: 0.0,
                    });

                    if let Some(handle) = node.get_handle()
                        && let Ok(entry) = directional_light_arena.get_mut(handle)
                    {
                        let directional_light = &mut entry.item;

                        let rotate_x = Quaternion::new(vec3::RIGHT, -PI / 4.0);

                        let rotate_y = Quaternion::new(vec3::UP, PI / 2.0);

                        directional_light.set_direction(rotate_x * rotate_y);

                        directional_light.intensities = vec3::ONES * 0.5;

                        directional_light.enable_shadow_maps(384, 40.0, resources.clone());
                    }

                    Ok(())
                }
                SceneNodeType::Camera => {
                    if let Some(handle) = node.get_handle()
                        && let Ok(entry) = camera_arena.get_mut(handle)
                    {
                        let camera = &mut entry.item;

                        camera.movement_speed = 2.0;

                        camera.look_vector.set_position(Vec3 {
                            x: 4.5,
                            y: 3.5,
                            z: -4.5,
                        });

                        camera.set_projection_z_far(20.0);
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        },
    )?;

    // Load an object to place on the ground.

    let LoadObjResult(_backpack_geometry, backpack_meshes) = load_obj(
        "./data/obj/suzanne.obj",
        material_arena,
        texture_u8_arena,
        Some(ProcessGeometryFlags::empty() | ProcessGeometryFlags::CENTER),
    );

    for entry in material_arena.entries.iter_mut().flatten() {
        let material = &mut entry.item;

        material.roughness = 1.0;
        material.metallic = 0.0;
        material.metallic_map = material.specular_exponent_map;

        material.load_all_maps(texture_u8_arena, rendering_context)?;
    }

    // Assign the meshes to entities

    for mesh in backpack_meshes {
        let material_handle = mesh.material;

        let mesh_handle = mesh_arena.insert(mesh.to_owned());

        let entity_handle = entity_arena.insert(Entity::new(mesh_handle, material_handle));

        scene.root.add_child(SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        ))?;
    }

    Ok((scene, shader_context))
}
