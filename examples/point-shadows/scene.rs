#![allow(clippy::result_unit_err)]

use std::{f32::consts::PI, rc::Rc};

use cairo::{
    color,
    entity::Entity,
    material::Material,
    mesh::{primitive::cube, Mesh},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, attenuation::LIGHT_ATTENUATION_RANGE_50_UNITS,
            directional_light::DirectionalLight, point_light::PointLight,
        },
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    transform::Transform3D,
    vec::vec3::{self, Vec3},
};

#[allow(clippy::too_many_arguments)]
pub fn make_scene(
    resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    point_light_arena: &mut Arena<PointLight>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    // Move out default camera.

    if let Some(handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Camera)
        .unwrap()
    {
        if let Ok(entry) = camera_arena.get_mut(&handle) {
            let camera = &mut entry.item;

            camera.look_vector.set_position(Vec3 {
                x: 25.0,
                y: 25.0,
                z: 25.0,
            });

            camera.look_vector.set_target(Default::default());
        }
    }

    // Add point lights to our scene.

    for (index, color) in [color::RED, color::GREEN].iter().enumerate() {
        let point_light_node = {
            let point_light = {
                let mut light = PointLight::new();

                light.intensities = (color.to_vec3() / 255.0) * 10.0;

                light.set_attenuation(LIGHT_ATTENUATION_RANGE_50_UNITS);

                light.enable_shadow_maps(128, 100.0, resources.clone());

                light
            };

            let point_light_handle = point_light_arena.insert(point_light);

            let mut transform = Transform3D::default();

            let y = 12.0 + index as f32 * 2.0;

            let factor = (y - 5.0) / 2.0;

            transform.set_translation(Vec3 {
                x: 10.0 * (PI / 2.0 * factor).sin(),
                y,
                z: 10.0 * (PI / 2.0 * factor).cos(),
            });

            SceneNode::new(
                SceneNodeType::PointLight,
                transform,
                Some(point_light_handle),
            )
        };

        scene.root.add_child(point_light_node).unwrap();
    }

    // Add a ground plane to our scene.

    let mut plane_entity_node = {
        let mesh = cube::generate(50.0, 1.0, 50.0);

        let mesh_handle = mesh_arena.insert(mesh);

        let plane_material_handle = material_arena.insert(Material {
            name: "plane".to_string(),
            albedo: vec3::ONES,
            roughness: 0.0,
            ..Default::default()
        });

        let entity = Entity::new(mesh_handle, Some(plane_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let transform = Transform3D::default();

        SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle))
    };

    // Add cubes to our scene.

    let green_material_handle = material_arena.insert(Material {
        name: "green".to_string(),
        albedo: color::GREEN.to_vec3() / 255.0,
        roughness: 0.0,
        ..Default::default()
    });

    static CUBE_ROWS: usize = 3;
    static CUBE_COLUMNS: usize = 3;

    static CUBE_WIDTH: f32 = 2.0;
    static CUBE_SPACING: f32 = 4.0;

    let cube_entity_handle = {
        let mesh = cube::generate(CUBE_WIDTH, CUBE_WIDTH, CUBE_WIDTH);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(green_material_handle));

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
                    y: 1.5 + 0.0 * (x * z) as f32,
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
