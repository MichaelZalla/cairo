use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh::{
        primitive::{cube, plane},
        Mesh,
    },
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    vec::vec3::Vec3,
};

pub(crate) fn make_scene(
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
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    // Add a textured ground plane to our scene.

    let checkerboard_material_handle = {
        let checkerboard_material = {
            let mut material = Material::new("checkerboard".to_string());

            let mut albedo_map = TextureMap::new(
                "./assets/textures/checkerboard.jpg",
                TextureMapStorageFormat::Index8(0),
            );

            // Checkerboard material

            albedo_map.sampling_options.wrapping = TextureMapWrapping::Repeat;

            albedo_map.load(rendering_context)?;

            let albedo_map_handle = texture_u8_arena.insert(albedo_map);

            material.albedo_map = Some(albedo_map_handle);

            material
        };

        material_arena.insert(checkerboard_material)
    };

    let mut plane_entity_node = {
        let mut mesh = plane::generate(80.0, 80.0, 8, 8);

        mesh.material = Some(checkerboard_material_handle);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(checkerboard_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            z: 3.0,
            y: -3.0,
            ..Default::default()
        });

        node
    };

    // Add a container (cube) to our scene.

    let emissive_material_handle = {
        let emissive_material = {
            let mut material = Material::new("emissive".to_string());

            material.albedo_map = Some(texture_u8_arena.insert(TextureMap::new(
                "./examples/post-effects/assets/lava.png",
                TextureMapStorageFormat::RGB24,
            )));

            material.emissive_color_map = Some(texture_u8_arena.insert(TextureMap::new(
                "./examples/post-effects/assets/lava_emissive.png",
                TextureMapStorageFormat::Index8(0),
            )));

            material
                .load_all_maps(texture_u8_arena, rendering_context)
                .unwrap();

            material
        };

        material_arena.insert(emissive_material)
    };

    let cube_entity_node = {
        let mesh = cube::generate(2.0, 2.0, 2.0);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(emissive_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            y: 3.0,
            ..Default::default()
        });

        node
    };

    plane_entity_node.add_child(cube_entity_node)?;

    scene.root.add_child(plane_entity_node)?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut light = PointLight::new();

        light.intensities = Vec3::ones() * 0.8;

        let light_handle = point_light_arena.insert(light);

        SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(light_handle),
        )
    };

    scene.root.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut light = SpotLight::new();

        light.intensities = Vec3::ones() * 0.1;

        light.look_vector.set_position(Vec3 {
            y: 30.0,
            ..light.look_vector.get_position()
        });

        let light_handle = spot_light_arena.insert(light);

        SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(light_handle),
        )
    };

    scene.root.add_child(spot_light_node)?;

    Ok((scene, shader_context))
}