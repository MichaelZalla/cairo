use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh::{self, Mesh},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    transform::Transform3D,
    vec::vec3::Vec3,
};

#[allow(clippy::too_many_arguments)]
pub fn make_scene(
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
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
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
        let mut mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);

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
        let mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

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
        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.8;

        let point_light_handle = point_light_arena.insert(point_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 0.0,
            y: 6.0,
            z: 0.0,
        });

        SceneNode::new(
            SceneNodeType::PointLight,
            transform,
            Some(point_light_handle),
        )
    };

    scene.root.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut spot_light = SpotLight::new();

        spot_light.intensities = Vec3::ones() * 0.1;

        let spot_light_handle = spot_light_arena.insert(spot_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 0.0,
            y: 30.0,
            z: 0.0,
        });

        SceneNode::new(SceneNodeType::SpotLight, transform, Some(spot_light_handle))
    };

    scene.root.add_child(spot_light_node)?;

    Ok((scene, shader_context))
}
