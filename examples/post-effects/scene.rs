use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh,
    scene::{
        graph::SceneGraph,
        light::{PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    vec::vec3::Vec3,
};
use uuid::Uuid;

pub fn make_scene(
    resources: &SceneResources,
    scene: &mut SceneGraph,
    rendering_context: &ApplicationRenderingContext,
) -> Result<(), String> {
    // Add a textured ground plane to our scene.

    let checkerboard_material_handle = {
        let mut materials = resources.material.borrow_mut();

        let checkerboard_material = {
            let mut material = Material::new("checkerboard".to_string());

            let mut albedo_map = TextureMap::new(
                "./assets/textures/checkerboard.jpg",
                TextureMapStorageFormat::Index8(0),
            );

            // Checkerboard material

            albedo_map.sampling_options.wrapping = TextureMapWrapping::Repeat;

            albedo_map.load(rendering_context)?;

            let albedo_map_handle = resources
                .texture_u8
                .borrow_mut()
                .insert(Uuid::new_v4(), albedo_map);

            material.albedo_map = Some(albedo_map_handle);

            material
        };

        materials.insert(Uuid::new_v4(), checkerboard_material)
    };

    let mut plane_entity_node = {
        let mut mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);

        mesh.material = Some(checkerboard_material_handle);

        let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

        let entity = Entity::new(mesh_handle, Some(checkerboard_material_handle));

        let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

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

    let emissive_material_handle = {
        let mut materials = resources.material.borrow_mut();

        let emissive_material = {
            let mut material = Material::new("emissive".to_string());

            material.albedo_map = Some(resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new(
                    "./examples/post-effects/assets/lava.png",
                    TextureMapStorageFormat::RGB24,
                ),
            ));

            material.emissive_color_map = Some(resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new(
                    "./examples/post-effects/assets/lava_emissive.png",
                    TextureMapStorageFormat::Index8(0),
                ),
            ));

            material
                .load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)
                .unwrap();

            material
        };

        materials.insert(Uuid::new_v4(), emissive_material)
    };

    let cube_entity_node = {
        let mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

        let mesh_handle = resources.mesh.borrow_mut().insert(Uuid::new_v4(), mesh);

        let entity = Entity::new(mesh_handle, Some(emissive_material_handle));

        let entity_handle = resources.entity.borrow_mut().insert(Uuid::new_v4(), entity);

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

    plane_entity_node.add_child(cube_entity_node)?;

    scene.root.add_child(plane_entity_node)?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut light = PointLight::new();

        light.intensities = Vec3::ones() * 0.8;

        let light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), light);

        SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(light_handle),
        )
    };

    scene.root.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut light = SpotLight::new();

        light.intensities = Vec3::ones() * 0.1;

        light.look_vector.set_position(Vec3 {
            y: 30.0,
            ..light.look_vector.get_position()
        });

        let light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), light);

        SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(light_handle),
        )
    };

    scene.root.add_child(spot_light_node)?;

    Ok(())
}
