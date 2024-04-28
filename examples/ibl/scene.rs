use cairo::{
    entity::Entity,
    mesh,
    scene::{
        camera::Camera,
        context::SceneContext,
        environment::Environment,
        light::{AmbientLight, DirectionalLight},
        node::{SceneNode, SceneNodeType},
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};
use uuid::Uuid;

pub fn make_cube_scene(camera_aspect_ratio: f32) -> Result<SceneContext, String> {
    let scene_context: SceneContext = Default::default();

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        let cube_mesh = mesh::primitive::cube::generate(1.0, 1.0, 1.0);

        let environment: Environment = Default::default();

        // Create resource handles from our arenas.

        let cube_entity_handle = {
            let cube_mesh_handle = resources
                .mesh
                .borrow_mut()
                .insert(Uuid::new_v4(), cube_mesh);

            resources
                .entity
                .borrow_mut()
                .insert(Uuid::new_v4(), Entity::new(cube_mesh_handle, None))
        };

        let camera_handle = {
            let camera: Camera = Camera::from_perspective(
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                vec3::FORWARD,
                75.0,
                camera_aspect_ratio,
            );

            resources.camera.borrow_mut().insert(Uuid::new_v4(), camera)
        };

        let environment_handle = resources
            .environment
            .borrow_mut()
            .insert(Uuid::new_v4(), environment);

        let ambient_light_handle = {
            let ambient_light = AmbientLight {
                intensities: Vec3::ones() * 0.5,
            };

            resources
                .ambient_light
                .borrow_mut()
                .insert(Uuid::new_v4(), ambient_light)
        };

        let directional_light_handle = {
            let directional_light = DirectionalLight {
                intensities: Vec3::ones() * 0.5,
                direction: Vec4 {
                    x: 0.25,
                    y: -1.0,
                    z: -0.25,
                    w: 1.0,
                }
                .as_normal(),
            };

            resources
                .directional_light
                .borrow_mut()
                .insert(Uuid::new_v4(), directional_light)
        };

        let mut environment_node = SceneNode::new(
            SceneNodeType::Environment,
            Default::default(),
            Some(environment_handle),
        );

        environment_node.add_child(SceneNode::new(
            SceneNodeType::AmbientLight,
            Default::default(),
            Some(ambient_light_handle),
        ))?;

        environment_node.add_child(SceneNode::new(
            SceneNodeType::DirectionalLight,
            Default::default(),
            Some(directional_light_handle),
        ))?;

        scene.root.add_child(environment_node)?;

        let cube_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(cube_entity_handle),
        );

        scene.root.add_child(cube_entity_node)?;

        let camera_node = SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(camera_handle),
        );

        scene.root.add_child(camera_node)?;
    }

    Ok(scene_context)
}
