use std::rc::Rc;

use cairo::{
    color,
    entity::Entity,
    material::Material,
    mesh::{primitive::cube, Mesh},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_cube_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{ambient_light::AmbientLight, directional_light::DirectionalLight},
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    transform::Transform3D,
    vec::vec3::{self, Vec3},
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_scene(
    resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_cube_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
        mesh_arena,
        material_arena,
        entity_arena,
    )?;

    // Dim our ambient light.

    if let Some(ambient_light_handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::AmbientLight)
        .as_ref()
        .unwrap()
        && let Ok(entry) = ambient_light_arena.get_mut(ambient_light_handle)
    {
        let ambient_light = &mut entry.item;

        ambient_light.intensities = vec3::ONES * 0.005;
    }

    // Brighten our directional light.

    if let Some(directional_light_handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::DirectionalLight)
        .as_ref()
        .unwrap()
        && let Ok(entry) = directional_light_arena.get_mut(directional_light_handle)
    {
        let directional_light = &mut entry.item;

        directional_light.intensities = vec3::ONES * 0.6;

        directional_light.enable_shadow_maps(1024, 48.0, resources.clone());
    }

    // Move our default camera.

    if let Some(handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Camera)
        .unwrap()
        && let Ok(entry) = camera_arena.get_mut(&handle)
    {
        let camera = &mut entry.item;

        camera.look_vector.set_position(Vec3 {
            x: 0.0,
            y: 12.0,
            z: -36.0,
        });

        camera.look_vector.set_target(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });

        camera.set_projection_z_far(150.0);
    }

    // Add a ground plane to our scene.

    let mut plane_entity_node = {
        let entity = {
            let mesh = cube::generate(100.0, 1.0, 100.0);

            let mesh_handle = mesh_arena.insert(mesh);

            let material_handle = material_arena.insert(Material {
                name: "plane".to_string(),
                albedo: vec3::ONES,
                roughness: 0.0,
                ..Default::default()
            });

            Entity::new(mesh_handle, Some(material_handle))
        };

        let entity_handle = entity_arena.insert(entity);

        let transform = Transform3D::default();

        SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle))
    };

    // Add cubes to our scene.

    static CUBE_ROWS: usize = 8;
    static CUBE_COLUMNS: usize = 8;

    static CUBE_WIDTH: f32 = 2.0;
    static CUBE_SPACING: f32 = 4.0;

    let cube_entity_handle = {
        let cube_mesh = cube::generate(CUBE_WIDTH, CUBE_WIDTH, CUBE_WIDTH);

        let cube_mesh_handle = mesh_arena.insert(cube_mesh);

        let cube_material_handle = material_arena.insert(Material {
            name: "red".to_string(),
            albedo: color::RED.to_vec3() / 255.0,
            roughness: 0.0,
            ..Default::default()
        });

        let entity = Entity::new(cube_mesh_handle, Some(cube_material_handle));

        entity_arena.insert(entity)
    };

    for x in 0..CUBE_COLUMNS {
        for z in 0..CUBE_ROWS {
            let cube_entity_node = {
                let mut node = SceneNode::new(
                    SceneNodeType::Entity,
                    Default::default(),
                    Some(cube_entity_handle),
                );

                node.get_transform_mut().set_translation(Vec3 {
                    x: -(CUBE_COLUMNS as f32 * CUBE_WIDTH
                        + (CUBE_COLUMNS - 1) as f32 * CUBE_SPACING)
                        / 2.0
                        + x as f32 * (CUBE_WIDTH + CUBE_SPACING),
                    z: -(CUBE_ROWS as f32 * CUBE_WIDTH + (CUBE_ROWS - 1) as f32 * CUBE_SPACING)
                        / 2.0
                        + z as f32 * (CUBE_WIDTH + CUBE_SPACING),
                    y: 1.5,
                });

                node
            };

            plane_entity_node.add_child(cube_entity_node).unwrap();
        }
    }

    // Add the ground plane to our scene.

    scene.root.add_child(plane_entity_node).unwrap();

    Ok((scene, shader_context))
}
