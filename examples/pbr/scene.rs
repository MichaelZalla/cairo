use cairo::{
    color,
    entity::Entity,
    material::Material,
    mesh::obj::load::load_obj,
    scene::{
        context::{utils::make_empty_scene, SceneContext},
        light::PointLight,
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    transform::Transform3D,
    vec::vec3::Vec3,
};

use uuid::Uuid;

pub fn make_sphere_grid_scene(
    camera_aspect_ratio: f32,
) -> Result<(SceneContext, ShaderContext), String> {
    let (scene_context, shader_context) = make_empty_scene(camera_aspect_ratio)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Move the camera backwards.

        {
            for entry in resources
                .camera
                .borrow_mut()
                .entries
                .as_mut_slice()
                .iter_mut()
                .flatten()
            {
                let camera = &mut entry.item;

                camera.look_vector.set_position(Vec3 {
                    z: -16.0,
                    ..Default::default()
                });
            }
        }

        // Generate a 2x2 grid of point lights.

        for grid_index_x in 0..4 {
            let mut light = PointLight::new();

            light.position = Vec3 {
                x: -8.0 + 4.0 * grid_index_x as f32,
                y: 4.0,
                z: -3.0,
            };

            light.intensities = Vec3::ones() * 1.0;

            light.constant_attenuation = 1.0;
            light.linear_attenuation = 0.09;
            light.quadratic_attenuation = 0.032;

            let point_light_handle = resources
                .point_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

            let point_light_node = SceneNode::new(
                SceneNodeType::PointLight,
                Default::default(),
                Some(point_light_handle),
            );

            scene.root.add_child(point_light_node)?;
        }

        let result = load_obj(
            "./examples/pbr/assets/sphere.obj",
            &mut resources.material.borrow_mut(),
            &mut resources.texture_u8.borrow_mut(),
        );

        let _geometry = result.0;
        let meshes = result.1;

        let mesh = meshes[1].to_owned();

        let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

        // Generate a grid of mesh instances.

        static GRID_ROWS: usize = 5;
        static GRID_COLUMNS: usize = 5;
        static SPACING: f32 = 1.0;

        static GRID_HEIGHT: f32 = GRID_ROWS as f32 + (GRID_ROWS as f32 - 1.0) * SPACING;
        static GRID_WIDTH: f32 = GRID_COLUMNS as f32 + (GRID_COLUMNS as f32 - 1.0) * SPACING;

        let base_transform: Transform3D = Default::default();

        for grid_index_y in 0..GRID_ROWS {
            let alpha_y = grid_index_y as f32 / (GRID_ROWS as f32 - 1.0);

            for grid_index_x in 0..GRID_COLUMNS {
                let alpha_x = grid_index_x as f32 / (GRID_COLUMNS as f32 - 1.0);

                let name = format!("instance_x{}_y{}", grid_index_x, grid_index_y).to_string();

                let material = Material {
                    name,
                    albedo: color::RED.to_vec3() / 255.0,
                    roughness: (alpha_x * 0.75).max(0.075),
                    metallic: alpha_y,
                    sheen: 0.0,
                    clearcoat_thickness: 0.0,
                    clearcoat_roughness: 0.0,
                    anisotropy: 0.0,
                    anisotropy_rotation: 0.0,
                    ..Default::default()
                };

                let material_handle = resources
                    .material
                    .borrow_mut()
                    .insert(Uuid::new_v4(), material);

                let entity = Entity::new(mesh_handle, Some(material_handle));

                let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

                let mut transform = base_transform;

                transform.set_translation(Vec3 {
                    x: -GRID_WIDTH / 2.0 + (GRID_WIDTH * alpha_x),
                    y: -GRID_HEIGHT / 2.0 + (GRID_HEIGHT * alpha_y),
                    z: 0.0,
                });

                let entity_node =
                    SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle));

                scene.root.add_child(entity_node)?;
            }
        }
    }

    Ok((scene_context, shader_context))
}
