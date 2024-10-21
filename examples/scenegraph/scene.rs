use cairo::{
    app::context::ApplicationRenderingContext,
    color,
    entity::Entity,
    material::Material,
    mesh::{
        primitive::{cube, plane},
        Mesh,
    },
    resource::{arena::Arena, handle::Handle},
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
        skybox::Skybox,
    },
    shader::context::ShaderContext,
    texture::{
        cubemap::CubeMap,
        map::{TextureMap, TextureMapStorageFormat},
    },
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
    cubemap_u8_arena: &mut Arena<CubeMap>,
    skybox_arena: &mut Arena<Skybox>,
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

    // Add a textured ground plane to our scene.

    let checkerboard_material_handle: Handle;

    {
        let checkerboard_material = {
            let mut material = Material::new("checkerboard".to_string());

            let mut checkerboard_albedo_map = TextureMap::new(
                "./assets/textures/checkerboard.jpg",
                TextureMapStorageFormat::Index8(0),
            );

            checkerboard_albedo_map.load(rendering_context)?;

            let checkerboard_albedo_map_handle = texture_u8_arena.insert(checkerboard_albedo_map);

            material.albedo_map = Some(checkerboard_albedo_map_handle);

            material
        };

        checkerboard_material_handle = material_arena.insert(checkerboard_material);
    }

    let mut plane_entity_node = {
        let mut plane_mesh = plane::generate(80.0, 80.0, 8, 8);

        plane_mesh.material = Some(checkerboard_material_handle);

        let plane_mesh_handle = mesh_arena.insert(plane_mesh);

        let plane_entity = Entity::new(plane_mesh_handle, Some(checkerboard_material_handle));

        let plane_entity_handle = entity_arena.insert(plane_entity);

        SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(plane_entity_handle),
        )
    };

    // Add some cubes to our scene.

    let mut red_cube_material = Material::new("red".to_string());
    red_cube_material.albedo = color::RED.to_vec3() / 255.0;

    let mut green_cube_material = Material::new("green".to_string());
    green_cube_material.albedo = color::GREEN.to_vec3() / 255.0;

    let mut blue_cube_material = Material::new("blue".to_string());
    blue_cube_material.albedo = color::BLUE.to_vec3() / 255.0;

    let (red_cube_material_handle, green_cube_material_handle, blue_cube_material_handle) = (
        material_arena.insert(red_cube_material),
        material_arena.insert(green_cube_material),
        material_arena.insert(blue_cube_material),
    );

    // Blue cube (1x1)

    let cube_mesh = cube::generate(3.0, 3.0, 3.0);

    let blue_cube_entity_node = {
        let mut mesh = cube_mesh.clone();

        mesh.object_name = Some("blue_cube".to_string());

        mesh.material = Some(blue_cube_material_handle);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(blue_cube_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        let mut scale = *(node.get_transform().scale());
        let mut translate = *(node.get_transform().translation());

        scale *= 2.0 / 3.0;
        translate.y = 4.0;

        node.get_transform_mut().set_translation(translate);
        node.get_transform_mut().set_scale(scale);

        node
    };

    // Green cube (2x2)

    let mut green_cube_entity_node = {
        let mut mesh = cube_mesh.clone();

        mesh.object_name = Some("green_cube".to_string());

        mesh.material = Some(green_cube_material_handle);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(green_cube_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        let mut scale = *(node.get_transform().scale());
        let mut translate = *(node.get_transform().translation());

        scale *= 2.0 / 3.0;
        translate.y = 4.0;

        node.get_transform_mut().set_translation(translate);
        node.get_transform_mut().set_scale(scale);

        node
    };

    // Red cube (3x3)

    let mut red_cube_entity_node = {
        let mut mesh = cube_mesh.clone();

        mesh.object_name = Some("red_cube".to_string());

        mesh.material = Some(red_cube_material_handle);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(red_cube_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        let mut translate = *(node.get_transform().translation());

        translate.y = 3.0;

        node.get_transform_mut().set_translation(translate);

        node
    };

    // Add a skybox to our scene.

    let skybox_node = {
        let mut skybox_cubemap = CubeMap::new(
            [
                "examples/skybox/assets/sides/front.jpg",
                "examples/skybox/assets/sides/back.jpg",
                "examples/skybox/assets/sides/top.jpg",
                "examples/skybox/assets/sides/bottom.jpg",
                "examples/skybox/assets/sides/left.jpg",
                "examples/skybox/assets/sides/right.jpg",
            ],
            TextureMapStorageFormat::RGB24,
        );

        skybox_cubemap.load(rendering_context).unwrap();

        let skybox_cubemap_handle = cubemap_u8_arena.insert(skybox_cubemap);

        let skybox = Skybox {
            is_hdr: false,
            radiance: Some(skybox_cubemap_handle),
            irradiance: None,
            specular_prefiltered_environment: None,
        };

        let skybox_handle = skybox_arena.insert(skybox);

        SceneNode::new(
            SceneNodeType::Skybox,
            Default::default(),
            Some(skybox_handle),
        )
    };

    for node in scene.root.children_mut().as_mut().unwrap() {
        if *node.get_type() == SceneNodeType::Environment {
            node.add_child(skybox_node)?;

            break;
        }
    }

    // Add a point light to our scene.

    let point_light_node = {
        let point_light = PointLight::new();

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

    plane_entity_node.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let mut spot_light_node = {
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

    // Add a spot light as a child of the green cube.

    spot_light_node.get_transform_mut().set_translation(Vec3 {
        x: 0.0,
        y: 10.0,
        z: 0.0,
    });

    green_cube_entity_node.add_child(spot_light_node)?;

    // Add the blue cube as a child of the green cube.

    green_cube_entity_node.add_child(blue_cube_entity_node)?;

    // Add the green cube as a child of the red cube.

    red_cube_entity_node.add_child(green_cube_entity_node)?;

    // Add the red cube as a child of the ground plane.

    plane_entity_node.add_child(red_cube_entity_node)?;

    scene.root.add_child(plane_entity_node)?;

    // Adjust our scene's default camera.

    if let Some(camera_handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Camera)
        .unwrap()
    {
        if let Ok(entry) = camera_arena.get_mut(&camera_handle) {
            let camera = &mut entry.item;

            camera.look_vector.set_position(Vec3 {
                x: 0.0,
                y: 24.0,
                z: -32.0,
            });

            camera.look_vector.set_target_position(Default::default());
        }
    }

    Ok((scene, shader_context))
}
