use cairo::{
    color,
    entity::Entity,
    material::Material,
    mesh::{self, obj::load::load_obj},
    scene::{
        camera::Camera,
        context::SceneContext,
        environment::Environment,
        light::{AmbientLight, DirectionalLight, PointLight},
        node::{SceneNode, SceneNodeType},
    },
    transform::Transform3D,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use uuid::Uuid;

pub fn make_empty_scene(camera_aspect_ratio: f32) -> Result<SceneContext, String> {
    let scene_context: SceneContext = Default::default();

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        let environment: Environment = Default::default();

        // Create resource handles from our arenas.

        let camera_handle = {
            let camera: Camera = Camera::from_perspective(
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: -3.0,
                },
                vec3::FORWARD,
                75.0,
                camera_aspect_ratio,
            );

            resources.camera.borrow_mut().insert(Uuid::new_v4(), camera)
        };

        let environment_handle = resources
            .environment
            .borrow_mut()
            .insert(Uuid::new_v4(), environment);

        let ambient_light_handle = {
            let ambient_light = AmbientLight {
                intensities: Vec3::ones() * 0.5,
            };

            resources
                .ambient_light
                .borrow_mut()
                .insert(Uuid::new_v4(), ambient_light)
        };

        let directional_light_handle = {
            let directional_light = DirectionalLight {
                intensities: Vec3::ones() * 0.5,
                direction: Vec4 {
                    x: 0.25,
                    y: -1.0,
                    z: -0.25,
                    w: 1.0,
                }
                .as_normal(),
            };

            resources
                .directional_light
                .borrow_mut()
                .insert(Uuid::new_v4(), directional_light)
        };

        let mut environment_node = SceneNode::new(
            SceneNodeType::Environment,
            Default::default(),
            Some(environment_handle),
        );

        environment_node.add_child(SceneNode::new(
            SceneNodeType::AmbientLight,
            Default::default(),
            Some(ambient_light_handle),
        ))?;

        environment_node.add_child(SceneNode::new(
            SceneNodeType::DirectionalLight,
            Default::default(),
            Some(directional_light_handle),
        ))?;

        scene.root.add_child(environment_node)?;

        let camera_node = SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(camera_handle),
        );

        scene.root.add_child(camera_node)?;
    }

    Ok(scene_context)
}

pub fn make_cube_scene(camera_aspect_ratio: f32) -> Result<SceneContext, String> {
    let scene_context = make_empty_scene(camera_aspect_ratio)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        let cube_mesh = mesh::primitive::cube::generate(1.0, 1.0, 1.0);

        let cube_entity_handle = {
            let cube_mesh_handle = resources
                .mesh
                .borrow_mut()
                .insert(Uuid::new_v4(), cube_mesh);

            resources
                .entity
                .borrow_mut()
                .insert(Uuid::new_v4(), Entity::new(cube_mesh_handle, None))
        };

        let cube_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(cube_entity_handle),
        );

        scene.root.add_child(cube_entity_node)?;
    }

    Ok(scene_context)
}

pub fn make_sphere_grid_scene(camera_aspect_ratio: f32) -> Result<SceneContext, String> {
    let scene_context = make_empty_scene(camera_aspect_ratio)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Generate a 2x2 grid of point lights.

        for grid_index_x in 0..4 {
            let mut light = PointLight::new();

            light.position = Vec3 {
                x: -8.0 + 4.0 * grid_index_x as f32,
                y: 4.0,
                z: -3.0,
            };

            light.intensities = Vec3::ones() * 15.0;
            light.specular_intensity = 20.0;

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
            &mut resources.texture.borrow_mut(),
        );

        let _geometry = result.0;
        let meshes = result.1;
        let mesh = meshes[1].to_owned();
        let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

        // Generate a grid of mesh instances.

        static GRID_ROWS: usize = 6;
        static GRID_COLUMNS: usize = 6;
        static SPACING: f32 = 1.0;

        static GRID_HEIGHT: f32 = GRID_ROWS as f32 + (GRID_ROWS as f32 - 1.0) * SPACING;
        static GRID_WIDTH: f32 = GRID_COLUMNS as f32 + (GRID_COLUMNS as f32 - 1.0) * SPACING;

        let base_transform: Transform3D = Default::default();

        for grid_index_y in 0..GRID_ROWS {
            let alpha_y = grid_index_y as f32 / (GRID_ROWS as f32 - 1.0);

            for grid_index_x in 0..GRID_COLUMNS {
                let alpha_x = grid_index_x as f32 / (GRID_COLUMNS as f32 - 1.0);

                let (material_name, material) = {
                    let name = format!("instance_x{}_y{}", grid_index_x, grid_index_y).to_string();

                    (
                        name.clone(),
                        Material {
                            name,
                            albedo: color::WHITE.to_vec3() / 255.0,
                            roughness: (alpha_x * 0.75).max(0.075),
                            metallic: alpha_y,
                            sheen: 0.0,
                            clearcoat_thickness: 0.0,
                            clearcoat_roughness: 0.0,
                            anisotropy: 0.0,
                            anisotropy_rotation: 0.0,
                            ..Default::default()
                        },
                    )
                };

                resources.material.borrow_mut().insert(material);

                let entity = Entity::new(mesh_handle, Some(material_name.clone()));

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

    Ok(scene_context)
}
