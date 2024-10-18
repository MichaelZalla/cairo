use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh::{primitive::cube, Mesh},
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
    texture::map::{TextureMap, TextureMapStorageFormat},
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

    // Brick wall material.

    let mut brick_material = Material::new("brick".to_string());

    brick_material.albedo_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/normal-map/assets/Brick_OldDestroyed_1k_d.tga",
        TextureMapStorageFormat::RGB24,
    )));

    brick_material.specular_exponent_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/normal-map/assets/Brick_OldDestroyed_1k_s.tga",
        TextureMapStorageFormat::Index8(0),
    )));

    brick_material.normal_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/normal-map/assets/Brick_OldDestroyed_1k_nY+.tga",
        TextureMapStorageFormat::RGB24,
    )));

    brick_material.load_all_maps(texture_u8_arena, rendering_context)?;

    let brick_material_handle = material_arena.insert(brick_material);

    // Add a brick wall to our scene.

    let brick_wall_mesh = cube::generate(1.5, 1.5, 1.5);

    let brick_wall_mesh_handle = mesh_arena.insert(brick_wall_mesh);

    let brick_wall_entity = Entity::new(brick_wall_mesh_handle, Some(brick_material_handle));

    let brick_wall_entity_handle = entity_arena.insert(brick_wall_entity);

    scene.root.add_child(SceneNode::new(
        SceneNodeType::Entity,
        Default::default(),
        Some(brick_wall_entity_handle),
    ))?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut light = PointLight::new();

        light.position.y = 0.0;
        light.position.z = -4.0;

        light.intensities = Vec3::ones() * 10.0;

        light.constant_attenuation = 1.0;
        light.linear_attenuation = 0.35;
        light.quadratic_attenuation = 0.44;

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
        let light = SpotLight::new();

        let light_handle = spot_light_arena.insert(light);

        SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(light_handle),
        )
    };

    scene.root.add_child(spot_light_node)?;

    //

    Ok((scene, shader_context))
}
