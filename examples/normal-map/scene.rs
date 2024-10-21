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
        light::{
            ambient_light::AmbientLight, attenuation::LightAttenuation,
            directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat},
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
        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 10.0;

        point_light.attenuation = LightAttenuation::new(1.0, 0.35, 0.44);

        let point_light_handle = point_light_arena.insert(point_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 0.0,
            y: 10.0,
            z: -4.0,
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
        let spot_light = SpotLight::new();

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

    //

    Ok((scene, shader_context))
}
