use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh::{Mesh, primitive::cube},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, attenuation::LIGHT_ATTENUATION_RANGE_13_UNITS,
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
    texture_u8_arena: &mut Arena<TextureMap>,
    rendering_context: &ApplicationRenderingContext,
    material_arena: &mut Arena<Material>,
    mesh_arena: &mut Arena<Mesh>,
    entity_arena: &mut Arena<Entity>,
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = {
        make_empty_scene(
            camera_arena,
            camera_aspect_ratio,
            environment_arena,
            ambient_light_arena,
            directional_light_arena,
        )
    }?;

    // Bricks material

    let mut brick_material = Material::new("brick".to_string());

    brick_material.albedo_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/displacement-map/assets/bricks2.jpg",
        TextureMapStorageFormat::RGB24,
    )));

    brick_material.normal_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/displacement-map/assets/bricks2_normal.jpg",
        TextureMapStorageFormat::RGB24,
    )));

    brick_material.displacement_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/displacement-map/assets/bricks2_disp.jpg",
        TextureMapStorageFormat::Index8(0),
    )));

    brick_material.displacement_scale = 0.05;

    brick_material.load_all_maps(texture_u8_arena, rendering_context)?;

    // Box material

    let mut box_material = Material::new("box".to_string());

    box_material.albedo_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/displacement-map/assets/wood.png",
        TextureMapStorageFormat::RGB24,
    )));

    box_material.normal_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/displacement-map/assets/toy_box_normal.png",
        TextureMapStorageFormat::RGB24,
    )));

    box_material.displacement_map = Some(texture_u8_arena.insert(TextureMap::new(
        "./examples/displacement-map/assets/toy_box_disp.png",
        TextureMapStorageFormat::Index8(0),
    )));

    box_material.displacement_scale = 0.05;

    box_material.load_all_maps(texture_u8_arena, rendering_context)?;

    // Collect materials

    let brick_material_handle = material_arena.insert(brick_material);
    let box_material_handle = material_arena.insert(box_material);

    // Add a brick wall to our scene.

    let mut brick_wall_entity_node = {
        let brick_wall_mesh = cube::generate(1.5, 1.5, 1.5);

        let brick_wall_mesh_handle = mesh_arena.insert(brick_wall_mesh);

        let brick_wall_entity = Entity::new(brick_wall_mesh_handle, Some(brick_material_handle));

        let brick_wall_entity_handle = entity_arena.insert(brick_wall_entity);

        SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(brick_wall_entity_handle),
        )
    };

    brick_wall_entity_node
        .get_transform_mut()
        .set_translation(Vec3 {
            x: -2.0,
            y: 0.0,
            z: 4.0,
        });

    scene.root.add_child(brick_wall_entity_node)?;

    // Add a wooden box to our scene.

    let wooden_box_entity_node = {
        let mesh = cube::generate(1.5, 1.5, 1.5);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(box_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 2.0,
            y: 0.0,
            z: 4.0,
        });

        SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle))
    };

    scene.root.add_child(wooden_box_entity_node)?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 10.0;

        point_light.set_attenuation(LIGHT_ATTENUATION_RANGE_13_UNITS);

        let point_light_handle = point_light_arena.insert(point_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 0.0,
            y: 0.0,
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

    Ok((scene, shader_context))
}
