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
        light::{
            ambient_light::AmbientLight, attenuation::LightAttenuation,
            directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    vec::vec3::Vec3,
};

use crate::MAX_POINT_LIGHT_INTENSITY;

pub(crate) fn make_scene(
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

    let cube_material_handle = {
        let container_material = {
            let mut material = Material::new("container".to_string());

            material.albedo_map = Some(texture_u8_arena.insert(TextureMap::new(
                "./examples/specular-map/assets/container2.png",
                TextureMapStorageFormat::RGB24,
            )));

            material.specular_exponent_map = Some(texture_u8_arena.insert(TextureMap::new(
                "./examples/specular-map/assets/container2_specular.png",
                TextureMapStorageFormat::Index8(0),
            )));

            material
                .load_all_maps(texture_u8_arena, rendering_context)
                .unwrap();

            material
        };

        material_arena.insert(container_material)
    };

    let cube_entity_node = {
        let mesh = cube::generate(2.0, 2.0, 2.0);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(cube_material_handle));

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

    // Add the container as a child of the ground plane.

    plane_entity_node.add_child(cube_entity_node)?;

    // Add the ground plane to our scene.

    scene.root.add_child(plane_entity_node)?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut light = PointLight::new();

        light.intensities = Vec3::ones() * MAX_POINT_LIGHT_INTENSITY;

        light.attenuation = LightAttenuation::new(1.0, 0.22, 0.2);

        let point_light_handle = point_light_arena.insert(light);

        SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        )
    };

    scene.root.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut spot_light = SpotLight::new();

        spot_light.intensities = Vec3::ones() * 0.0;

        spot_light.look_vector.set_position(Vec3 {
            y: 10.0,
            ..spot_light.look_vector.get_position()
        });

        let spot_light_handle = spot_light_arena.insert(spot_light);

        SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        )
    };

    scene.root.add_child(spot_light_node)?;

    Ok((scene, shader_context))
}
