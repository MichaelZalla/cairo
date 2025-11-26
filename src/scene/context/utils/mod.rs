use crate::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::{self, Mesh},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat},
    transform::Transform3D,
    vec::vec3::Vec3,
};

pub fn make_empty_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let mut scene: SceneGraph = Default::default();

    let mut shader_context: ShaderContext = Default::default();

    // Create resource handles from our arenas.

    let camera_handle = {
        let mut camera: Camera = Camera::perspective(
            Vec3 {
                x: 15.0,
                y: 5.0,
                z: -15.0,
            },
            Default::default(),
            75.0,
            camera_aspect_ratio,
        );

        camera.is_active = true;

        camera.update_shader_context(&mut shader_context);

        camera_arena.insert(camera)
    };

    let environment_handle = environment_arena.insert(Default::default());

    let environment_node = {
        let mut environment_node = SceneNode::new(
            SceneNodeType::Environment,
            Default::default(),
            Some(environment_handle),
        );

        let ambient_light_node = {
            let ambient_light_handle = {
                let ambient_light = AmbientLight {
                    intensities: Vec3::ones() * 0.15,
                };

                ambient_light_arena.insert(ambient_light)
            };

            let mut transform = Transform3D::default();

            transform.set_translation(Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            });

            let mut node = SceneNode::new(
                SceneNodeType::AmbientLight,
                transform,
                Some(ambient_light_handle),
            );

            node.name.replace("ambient_light_1".to_string());

            node
        };

        environment_node.add_child(ambient_light_node)?;

        let directional_light_node = {
            let directional_light = DirectionalLight::default();

            let directional_light_handle = directional_light_arena.insert(directional_light);

            let mut transform = Transform3D::default();

            transform.set_translation(Vec3 {
                x: 8.0,
                y: 10.0,
                z: -8.0,
            });

            let mut node = SceneNode::new(
                SceneNodeType::DirectionalLight,
                transform,
                Some(directional_light_handle),
            );

            node.name.replace("directional_light_1".to_string());

            node
        };

        environment_node.add_child(directional_light_node)?;

        environment_node
    };

    scene.root.add_child(environment_node)?;

    let camera_node = {
        let mut node = SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(camera_handle),
        );

        node.name.replace("camera_1".to_string());

        node
    };

    scene.root.add_child(camera_node)?;

    Ok((scene, shader_context))
}

pub fn make_cube_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    // Move the default camera.

    scene.root.visit_mut(
        SceneNodeGlobalTraversalMethod::DepthFirst,
        None,
        &mut |_current_depth: usize, _current_world_transform: Mat4, node: &mut SceneNode| {
            match node.get_type() {
                SceneNodeType::Camera => {
                    if let Some(handle) = node.get_handle()
                        && let Ok(entry) = camera_arena.get_mut(handle)
                    {
                        let camera = &mut entry.item;

                        camera.look_vector.set_position(Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: -4.0,
                        });
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        },
    )?;

    let cube_mesh = mesh::primitive::cube::generate(1.0, 1.0, 1.0);

    let cube_material = Material::new("cube_material".to_string());

    let cube_entity_handle = {
        let cube_mesh_handle = mesh_arena.insert(cube_mesh);

        let cube_material_handle = material_arena.insert(cube_material);

        let cube_entity = Entity::new(cube_mesh_handle, Some(cube_material_handle));

        entity_arena.insert(cube_entity)
    };

    let cube_entity_node = SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(cube_entity_handle),
    );

    scene.root.add_child(cube_entity_node)?;

    Ok((scene, shader_context))
}

pub fn make_textured_cube_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
    texture_u8_arena: &mut Arena<TextureMap>,
    rendering_context: &ApplicationRenderingContext,
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

    // Customize the cube material.

    let cube_material_handle = {
        let mut cube_material = {
            let cube_albedo_map_handle = texture_u8_arena.insert(TextureMap::new(
                "./data/obj/cobblestone.png",
                TextureMapStorageFormat::RGB24,
            ));

            Material {
                name: "cube".to_string(),
                albedo_map: Some(cube_albedo_map_handle),
                ..Default::default()
            }
        };

        cube_material.load_all_maps(texture_u8_arena, rendering_context)?;

        material_arena.insert(cube_material)
    };

    let cube_entity_handle = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Entity)
        .unwrap()
        .unwrap();

    match entity_arena.get_mut(&cube_entity_handle) {
        Ok(entry) => {
            let cube_entity = &mut entry.item;

            cube_entity.material = Some(cube_material_handle);
        }
        _ => panic!(),
    }

    // Add a point light to our scene.

    let mut point_light = PointLight::new();

    point_light.intensities = Vec3::ones() * 0.7;

    point_light.position = Vec3 {
        x: 0.0,
        y: 4.0,
        z: 0.0,
    };

    let point_light_handle = point_light_arena.insert(point_light);

    scene.root.add_child(SceneNode::new(
        SceneNodeType::PointLight,
        Default::default(),
        Some(point_light_handle),
    ))?;

    // Add a spot light to our scene.

    let spot_light = SpotLight::new();

    let spot_light_handle = spot_light_arena.insert(spot_light);

    scene.root.add_child(SceneNode::new(
        SceneNodeType::SpotLight,
        Default::default(),
        Some(spot_light_handle),
    ))?;

    Ok((scene, shader_context))
}
