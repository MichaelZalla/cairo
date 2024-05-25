#![allow(clippy::result_unit_err)]

use std::{cell::RefCell, rc::Rc};

use cairo::{
    buffer::framebuffer::Framebuffer,
    color,
    entity::Entity,
    material::Material,
    mesh,
    scene::{
        context::{utils::make_empty_scene, SceneContext},
        light::PointLight,
        node::{SceneNode, SceneNodeType},
    },
    texture::cubemap::CubeMap,
    vec::vec3::{self, Vec3},
};
use uuid::Uuid;

pub fn make_cubes_scene(
    camera_aspect_ratio: f32,
    point_shadow_map_framebuffer_rc: Rc<RefCell<Framebuffer>>,
) -> Result<SceneContext, ()> {
    let scene_context = make_empty_scene(camera_aspect_ratio).unwrap();

    {
        let resources = (*scene_context.resources).borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Move out default camera.

        if let Some(handle) = scene
            .root
            .find(&mut |node| *node.get_type() == SceneNodeType::Camera)
            .unwrap()
        {
            let mut camera_arena = resources.camera.borrow_mut();

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
            let mesh = mesh::primitive::cube::generate(50.0, 1.0, 50.0);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            resources.material.borrow_mut().insert(Material {
                name: "plane".to_string(),
                albedo: vec3::ONES,
                roughness: 0.0,
                ..Default::default()
            });

            let entity = Entity::new(mesh_handle, Some("plane".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

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

        resources.material.borrow_mut().insert(Material {
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
            let mesh = mesh::primitive::cube::generate(CUBE_WIDTH, CUBE_WIDTH, CUBE_WIDTH);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("green".to_string()));

            resources.entity.borrow_mut().insert(Uuid::new_v4(), entity)
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

                light.shadow_map = Some(CubeMap::<f32>::from_framebuffer(
                    &point_shadow_map_framebuffer_rc.borrow(),
                ));

                light.constant_attenuation = 1.0;
                light.linear_attenuation = 0.09;
                light.quadratic_attenuation = 0.032;

                light
            };

            let point_light_node = {
                let light = point_light.clone();

                let point_light_handle = resources
                    .point_light
                    .borrow_mut()
                    .insert(Uuid::new_v4(), light);

                SceneNode::new(
                    SceneNodeType::PointLight,
                    Default::default(),
                    Some(point_light_handle),
                )
            };

            scene.root.add_child(point_light_node).unwrap();
        }
    }

    Ok(scene_context)
}
