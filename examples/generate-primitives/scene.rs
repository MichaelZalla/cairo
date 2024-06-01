use uuid::Uuid;

use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh,
    scene::{
        camera::Camera,
        context::SceneContext,
        light::{PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
    },
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    vec::vec3::Vec3,
};

static LIGHT_GRID_SUBDIVISIONS: usize = 1;
static LIGHT_GRID_SIZE: f32 = 20.0;

pub static POINT_LIGHTS_COUNT: usize = (LIGHT_GRID_SUBDIVISIONS + 1).pow(2);

pub fn make_primitives_scene(
    aspect_ratio: f32,
    rendering_context: Option<&ApplicationRenderingContext>,
) -> Result<SceneContext, String> {
    let scene_context: SceneContext = Default::default();

    {
        let resources = scene_context.resources.borrow_mut();

        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a textured ground plane to our scene.

        {
            let mut materials = resources.material.borrow_mut();

            let checkerboard_material = {
                let mut material = Material::new("checkerboard".to_string());

                let mut albedo_map = TextureMap::new(
                    "./assets/textures/checkerboard.jpg",
                    TextureMapStorageFormat::Index8(0),
                );

                // Checkerboard material

                albedo_map.sampling_options.wrapping = TextureMapWrapping::Repeat;

                if let Some(ctx) = rendering_context {
                    albedo_map.load(ctx)?
                }

                // Pump up albedo value of the darkest pixels

                albedo_map.map(|r, g, b| {
                    if r < 4 && g < 4 && b < 4 {
                        return (18, 18, 18);
                    }
                    (r, g, b)
                })?;

                let albedo_map_handle = resources
                    .texture_u8
                    .borrow_mut()
                    .insert(Uuid::new_v4(), albedo_map);

                material.albedo_map = Some(albedo_map_handle);

                material
            };

            materials.insert(checkerboard_material);
        }

        let plane_entity_node = {
            let mut mesh = mesh::primitive::plane::generate(32.0, 32.0, 1, 1);

            mesh.material_name = Some("checkerboard".to_string());

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: -5.0,
                z: -5.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(plane_entity_node)?;

        // Add a cube to our scene.

        let cube_entity_node = {
            let mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: -4.0,
                y: 1.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(cube_entity_node)?;

        // Add a cone to our scene.

        let cone_entity_node = {
            let mesh = mesh::primitive::cone::generate(2.0, 2.0, 40);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: 0.0,
                y: 1.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(cone_entity_node)?;

        // Add a cylinder to our scene.

        let cylinder_entity_node = {
            let mesh = mesh::primitive::cylinder::generate(2.0, 2.0, 40);

            let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

            let entity = Entity::new(mesh_handle, Some("checkerboard".to_string()));

            let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

            let mut node = SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            );

            node.get_transform_mut().set_translation(Vec3 {
                x: 4.0,
                y: 1.0,
                ..Default::default()
            });

            node
        };

        scene.root.add_child(cylinder_entity_node)?;

        // Add point lights to our scene.

        let point_light_decal_material = {
            let mut material = Material::new("point_light_decal".to_string());

            material.alpha_map = Some(resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new(
                    "./assets/decals/point_light_small.png",
                    TextureMapStorageFormat::Index8(0),
                ),
            ));

            material.emissive_color_map = material.alpha_map;

            if let Some(ctx) = rendering_context {
                material.load_all_maps(&mut resources.texture_u8.borrow_mut(), ctx)?
            }

            material
        };

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(point_light_decal_material);
        }

        {
            let mut point_light_arena = resources.point_light.borrow_mut();

            for x in 0..(LIGHT_GRID_SUBDIVISIONS + 1) {
                for z in 0..(LIGHT_GRID_SUBDIVISIONS + 1) {
                    let mut light = PointLight::new();

                    light.position = Vec3 {
                        x: -(LIGHT_GRID_SIZE / 2.0)
                            + (x as f32 / LIGHT_GRID_SUBDIVISIONS as f32) * LIGHT_GRID_SIZE,
                        y: 1.0,
                        z: -(LIGHT_GRID_SIZE / 2.0)
                            + (z as f32 / LIGHT_GRID_SUBDIVISIONS as f32) * LIGHT_GRID_SIZE,
                    };

                    let point_light_handle = point_light_arena.insert(Uuid::new_v4(), light);

                    let point_light_node = SceneNode::new(
                        SceneNodeType::PointLight,
                        Default::default(),
                        Some(point_light_handle),
                    );

                    scene.root.add_child(point_light_node)?;
                }
            }
        }

        // Add a spot light to our scene.

        let spot_light_decal_material = {
            let mut material = Material::new("spot_light_decal".to_string());

            material.alpha_map = Some(resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new(
                    "./assets/decals/spot_light_small.png",
                    TextureMapStorageFormat::Index8(0),
                ),
            ));

            material.emissive_color_map = material.alpha_map;

            if let Some(ctx) = rendering_context {
                material.load_all_maps(&mut resources.texture_u8.borrow_mut(), ctx)?
            }

            material
        };

        {
            let mut materials = resources.material.borrow_mut();

            materials.insert(spot_light_decal_material);
        }

        let spot_light_node = {
            let mut spot_light = SpotLight::new();

            spot_light.look_vector.set_position(Vec3 {
                x: -6.0,
                y: 15.0,
                z: -6.0,
            });

            let spot_light_handle = resources
                .spot_light
                .borrow_mut()
                .insert(Uuid::new_v4(), spot_light);

            SceneNode::new(
                SceneNodeType::SpotLight,
                Default::default(),
                Some(spot_light_handle),
            )
        };

        scene.root.add_child(spot_light_node)?;

        // Add a second camera to our scene.

        let camera_node = {
            let camera = Camera::from_perspective(
                Vec3 {
                    x: 0.0,
                    y: 12.0,
                    z: -16.0,
                },
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.5,
                }
                .as_normal(),
                75.0,
                aspect_ratio,
            );

            let camera_handle = resources.camera.borrow_mut().insert(Uuid::new_v4(), camera);

            SceneNode::new(
                SceneNodeType::Camera,
                Default::default(),
                Some(camera_handle),
            )
        };

        scene.root.add_child(camera_node)?;
    }

    Ok(scene_context)
}
