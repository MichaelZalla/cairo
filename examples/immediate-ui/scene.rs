use cairo::{
    entity::Entity,
    material::Material,
    mesh::Mesh,
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_cube_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    transform::Transform3D,
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

    // Add a point light to our scene.

    let mut point_light = PointLight::new();

    point_light.intensities = Vec3::ones() * 0.7;

    let point_light_handle = point_light_arena.insert(point_light);

    let mut point_light_node_transform = Transform3D::default();

    point_light_node_transform.set_translation(Vec3 {
        x: 0.0,
        y: 6.0,
        z: 0.0,
    });

    scene.root.add_child(SceneNode::new(
        SceneNodeType::PointLight,
        point_light_node_transform,
        Some(point_light_handle),
    ))?;

    // Add a spot light to our scene.

    let mut spot_light = SpotLight::new();

    spot_light.look_vector.set_target_position(Vec3::default());

    let spot_light_handle = spot_light_arena.insert(spot_light);

    let mut spot_light_node_transform = Transform3D::default();

    spot_light_node_transform.set_translation(Vec3 {
        x: 0.0,
        y: 3.0,
        z: -3.0,
    });

    scene.root.add_child(SceneNode::new(
        SceneNodeType::SpotLight,
        spot_light_node_transform,
        Some(spot_light_handle),
    ))?;

    Ok((scene, shader_context))
}
