use cairo::{
    app::context::ApplicationRenderingContext,
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
    texture::{cubemap::CubeMap, map::TextureMapStorageFormat},
    vec::vec3::Vec3,
};

pub(crate) fn make_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
    rendering_context: &ApplicationRenderingContext,
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

    // Add a skybox to our scene.

    let skybox_node = {
        // Option 1. Cubemap as a set of 6 separate textures.

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

        // Option 2. Cubemap as one horizontal cross texture.

        // let mut skybox_cubemap = CubeMap::cross(
        //     "examples/skybox/assets/cross/horizontal_cross.png",
        //     TextureMapStorageFormat::RGB24,
        // );

        // Option 3. Cubemap as one vertical cross texture.

        // let mut skybox_cubemap = CubeMap::cross(
        //     "examples/skybox/assets/cross/vertical_cross.png",
        //     TextureMapStorageFormat::RGB24,
        // );

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

    let mut point_light = PointLight::new();

    point_light.intensities = Vec3::ones() * 0.4;

    let point_light_handle = point_light_arena.insert(point_light);

    scene.root.add_child(SceneNode::new(
        SceneNodeType::PointLight,
        Default::default(),
        Some(point_light_handle),
    ))?;

    // Add a spot light to our scene.

    let mut spot_light = SpotLight::new();

    spot_light.look_vector.set_position(Vec3 {
        y: 10.0,
        ..spot_light.look_vector.get_position()
    });

    let spot_light_handle = spot_light_arena.insert(spot_light);

    scene.root.add_child(SceneNode::new(
        SceneNodeType::SpotLight,
        Default::default(),
        Some(spot_light_handle),
    ))?;

    Ok((scene, shader_context))
}
