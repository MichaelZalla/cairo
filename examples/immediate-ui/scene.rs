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
        skybox::Skybox,
    },
    shader::context::ShaderContext,
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
    _point_light_arena: &mut Arena<PointLight>,
    _spot_light_arena: &mut Arena<SpotLight>,
    skybox_arena: &mut Arena<Skybox>,
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

    // Add a skybox to our scene.

    for node in scene.root.children_mut().as_mut().unwrap() {
        if node.is_type(SceneNodeType::Environment) {
            let skybox_node = {
                let skybox = Skybox {
                    is_hdr: true,
                    ..Default::default()
                };

                let skybox_handle = skybox_arena.insert(skybox);

                SceneNode::new(
                    SceneNodeType::Skybox,
                    Default::default(),
                    Some(skybox_handle),
                )
            };

            node.add_child(skybox_node)?;
        }
    }

    Ok((scene, shader_context))
}
