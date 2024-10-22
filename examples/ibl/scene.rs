use cairo::{
    color,
    entity::Entity,
    material::Material,
    mesh::{
        obj::load::{load_obj, LoadObjResult},
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
            directional_light::DirectionalLight, point_light::PointLight,
        },
        node::{SceneNode, SceneNodeType},
        skybox::Skybox,
    },
    shader::context::ShaderContext,
    texture::map::TextureMap,
    transform::Transform3D,
    vec::vec3::Vec3,
};

#[allow(clippy::too_many_arguments)]
pub fn make_sphere_grid_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    texture_u8_arena: &mut Arena<TextureMap>,
    point_light_arena: &mut Arena<PointLight>,
    skybox_arena: &mut Arena<Skybox>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    // Move out default camera.

    if let Some(handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Camera)
        .unwrap()
    {
        if let Ok(entry) = camera_arena.get_mut(&handle) {
            let camera = &mut entry.item;

            camera.look_vector.set_position(Vec3 {
                x: 0.0,
                y: 0.0,
                z: -20.0,
            });

            camera.look_vector.set_target_position(Default::default());
        }
    }

    // Generate a 2x2 grid of point lights.

    for grid_index_x in 0..4 {
        let mut light = PointLight::new();

        light.intensities = Vec3::ones() * 15.0;

        light.attenuation = LightAttenuation::new(1.0, 0.09, 0.032);

        let point_light_handle = point_light_arena.insert(light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: -8.0 + 4.0 * grid_index_x as f32,
            y: 4.0,
            z: -3.0,
        });

        let point_light_node = SceneNode::new(
            SceneNodeType::PointLight,
            transform,
            Some(point_light_handle),
        );

        scene.root.add_child(point_light_node)?;
    }

    let LoadObjResult(_geometry, meshes) = load_obj(
        "./examples/pbr/assets/sphere.obj",
        material_arena,
        texture_u8_arena,
        None,
    );

    let mesh = meshes[1].to_owned();
    let mesh_handle = mesh_arena.insert(mesh);

    // Generate a grid of mesh instances.

    static GRID_ROWS: usize = 6;
    static GRID_COLUMNS: usize = 6;
    static SPACING: f32 = 1.0;

    static GRID_HEIGHT: f32 = GRID_ROWS as f32 + (GRID_ROWS as f32 - 1.0) * SPACING;
    static GRID_WIDTH: f32 = GRID_COLUMNS as f32 + (GRID_COLUMNS as f32 - 1.0) * SPACING;

    let base_transform: Transform3D = Default::default();

    for grid_index_y in 0..GRID_ROWS {
        let alpha_y = grid_index_y as f32 / (GRID_ROWS as f32 - 1.0);

        for grid_index_x in 0..GRID_COLUMNS {
            let alpha_x = grid_index_x as f32 / (GRID_COLUMNS as f32 - 1.0);

            let name = format!("instance_x{}_y{}", grid_index_x, grid_index_y).to_string();

            let material = Material {
                name,
                albedo: color::WHITE.to_vec3() / 255.0,
                roughness: (alpha_x * 0.75).max(0.075),
                metallic: alpha_y,
                sheen: 0.0,
                clearcoat_thickness: 0.0,
                clearcoat_roughness: 0.0,
                anisotropy: 0.0,
                anisotropy_rotation: 0.0,
                ..Default::default()
            };

            let material_handle = material_arena.insert(material);

            let entity = Entity::new(mesh_handle, Some(material_handle));

            let entity_handle = entity_arena.insert(entity);

            let mut transform = base_transform;

            transform.set_translation(Vec3 {
                x: -GRID_WIDTH / 2.0 + (GRID_WIDTH * alpha_x),
                y: -GRID_HEIGHT / 2.0 + (GRID_HEIGHT * alpha_y),
                z: 0.0,
            });

            let entity_node = SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle));

            scene.root.add_child(entity_node)?;
        }
    }

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
