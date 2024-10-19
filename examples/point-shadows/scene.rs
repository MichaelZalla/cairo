#![allow(clippy::result_unit_err)]

use std::{cell::RefCell, rc::Rc};

use cairo::{
    buffer::framebuffer::Framebuffer,
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
        light::{AmbientLight, DirectionalLight, PointLight},
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::cubemap::CubeMap,
    vec::vec3::{self, Vec3},
};

pub fn make_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    point_light_arena: &mut Arena<PointLight>,
    cubemap_f32_arena: &mut Arena<CubeMap<f32>>,
    point_shadow_map_framebuffer_rc: Rc<RefCell<Framebuffer>>,
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
                x: 8.0,
                y: 12.0,
                z: 8.0,
            });

            camera.look_vector.set_target_position(Default::default());
        }
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

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            z: 3.0,
            ..Default::default()
        });

        node
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

    // Add a point light to our scene.

    for (index, color) in [
        color::WHITE, /*, color::RED, color::GREEN, color::BLUE*/
    ]
    .iter()
    .enumerate()
    {
        let point_light = {
            let mut light = PointLight::new();

            light.position.y = 8.0 + index as f32 * 2.0;

            light.intensities = (color.to_vec3() / 255.0) * 10.0;

            let shadow_map_handle = cubemap_f32_arena.insert(CubeMap::<f32>::from_framebuffer(
                &point_shadow_map_framebuffer_rc.borrow(),
            ));

            light.shadow_map = Some(shadow_map_handle);

            light.constant_attenuation = 1.0;
            light.linear_attenuation = 0.09;
            light.quadratic_attenuation = 0.032;

            light
        };

        let point_light_node = {
            let light = point_light.clone();

            let point_light_handle = point_light_arena.insert(light);

            SceneNode::new(
                SceneNodeType::PointLight,
                Default::default(),
                Some(point_light_handle),
            )
        };

        scene.root.add_child(point_light_node).unwrap();
    }

    Ok((scene, shader_context))
}
