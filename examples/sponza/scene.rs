use cairo::{
    app::context::ApplicationRenderingContext,
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
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
        skybox::Skybox,
    },
    shader::context::ShaderContext,
    texture::{
        cubemap::CubeMap,
        map::{TextureMap, TextureMapStorageFormat},
    },
    vec::vec3::{self, Vec3},
};

pub fn make_sponza_scene(
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
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    // Adjust our scene's default camera.

    if let Some(camera_handle) = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Camera)
        .unwrap()
    {
        if let Ok(entry) = camera_arena.get_mut(&camera_handle) {
            let camera = &mut entry.item;

            camera.look_vector.set_position(Vec3 {
                x: 1000.0,
                y: 300.0,
                z: 0.0,
            });

            camera
                .look_vector
                .set_target_position(camera.look_vector.get_position() + vec3::RIGHT * -1.0);

            camera.movement_speed = 300.0;

            camera.set_projection_z_far(10_000.0);
        }
    }

    // Add a point light to our scene.

    let point_light_node = {
        let mut light = PointLight::new();

        light.position = Vec3 {
            x: 300.0,
            y: 300.0,
            z: 0.0,
        };

        light.intensities = color::BLUE.to_vec3() / 255.0 * 5.0;

        light.constant_attenuation = 1.0;
        light.linear_attenuation = 0.0014;
        light.quadratic_attenuation = 0.000007;

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
        let mut light = SpotLight::new();

        light.look_vector.set_position(Vec3 {
            x: 300.0,
            y: 900.0,
            z: 0.0,
        });

        light.look_vector.set_target_position(Default::default());

        light.intensities = color::RED.to_vec3() / 255.0 * 3.0;

        light.constant_attenuation = 1.0;
        light.linear_attenuation = 0.007;
        light.quadratic_attenuation = 0.0002;

        let light_handle = spot_light_arena.insert(light);

        SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(light_handle),
        )
    };

    scene.root.add_child(spot_light_node)?;

    // Add a skybox to our scene.

    let skybox_node = {
        let mut skybox_cubemap: CubeMap = CubeMap::cross(
            "examples/skybox/assets/cross/skybox_texture.jpg",
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

    // Sponza meshes and materials

    let LoadObjResult(_atrium_geometry, atrium_meshes) = load_obj(
        "./examples/sponza/assets/sponza.obj",
        material_arena,
        texture_u8_arena,
    );

    for entry in material_arena.entries.iter_mut().flatten() {
        let material = &mut entry.item;

        material.roughness = 1.0;
        material.metallic = 0.0;
        material.metallic_map = material.specular_exponent_map;

        material.load_all_maps(texture_u8_arena, rendering_context)?;
    }

    // Assign the meshes to entities

    for mesh in atrium_meshes {
        let material_handle = mesh.material;

        let mesh_handle = mesh_arena.insert(mesh.to_owned());

        let entity_handle = entity_arena.insert(Entity::new(mesh_handle, material_handle));

        scene.root.add_child(SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        ))?;
    }

    Ok((scene, shader_context))
}
