use crate::{
    entity::Entity,
    material::Material,
    mesh::{self},
    scene::{
        camera::Camera,
        context::SceneContext,
        environment::Environment,
        light::{AmbientLight, DirectionalLight},
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use uuid::Uuid;

pub fn make_empty_scene(camera_aspect_ratio: f32) -> Result<(SceneContext, ShaderContext), String> {
    let scene_context: SceneContext = Default::default();

    let mut shader_context: ShaderContext = Default::default();

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        let environment: Environment = Default::default();

        // Create resource handles from our arenas.

        let camera_handle = {
            let mut camera: Camera = Camera::from_perspective(
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: -5.0,
                },
                vec3::FORWARD,
                75.0,
                camera_aspect_ratio,
            );

            camera.is_active = true;

            camera.recompute_world_space_frustum();

            shader_context.set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

            shader_context.set_view_inverse_transform(camera.get_view_inverse_transform());

            shader_context.set_projection(camera.get_projection());

            resources.camera.borrow_mut().insert(Uuid::new_v4(), camera)
        };

        let environment_handle = resources
            .environment
            .borrow_mut()
            .insert(Uuid::new_v4(), environment);

        let ambient_light_handle = {
            let ambient_light = AmbientLight {
                intensities: Vec3::ones() * 0.15,
            };

            resources
                .ambient_light
                .borrow_mut()
                .insert(Uuid::new_v4(), ambient_light)
        };

        let directional_light_handle = {
            let directional_light = DirectionalLight {
                intensities: Vec3::ones() * 0.15,
                direction: Vec4 {
                    x: 0.25,
                    y: -1.0,
                    z: -0.25,
                    w: 1.0,
                }
                .as_normal(),
                shadow_map_cameras: None,
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

        let camera_node = SceneNode::new(
            SceneNodeType::Camera,
            Default::default(),
            Some(camera_handle),
        );

        scene.root.add_child(camera_node)?;
    }

    Ok((scene_context, shader_context))
}

pub fn make_cube_scene(camera_aspect_ratio: f32) -> Result<(SceneContext, ShaderContext), String> {
    let (scene_context, shader_context) = make_empty_scene(camera_aspect_ratio)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        let cube_mesh = mesh::primitive::cube::generate(1.0, 1.0, 1.0);

        let cube_material = Material::new("cube_material".to_string());

        let cube_entity_handle = {
            let cube_mesh_handle = resources
                .mesh
                .borrow_mut()
                .insert(Uuid::new_v4(), cube_mesh);

            let cube_material_handle = resources
                .material
                .borrow_mut()
                .insert(Uuid::new_v4(), cube_material);

            resources.entity.borrow_mut().insert(
                Uuid::new_v4(),
                Entity::new(cube_mesh_handle, Some(cube_material_handle)),
            )
        };

        let cube_entity_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(cube_entity_handle),
        );

        scene.root.add_child(cube_entity_node)?;
    }

    Ok((scene_context, shader_context))
}
