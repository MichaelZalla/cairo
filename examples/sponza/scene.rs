use std::borrow::BorrowMut;

use uuid::Uuid;

use cairo::{
    app::context::ApplicationRenderingContext,
    buffer::framebuffer::Framebuffer,
    color,
    entity::Entity,
    mesh,
    scene::{
        context::{utils::make_empty_scene, SceneContext},
        light::{PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    vec::vec3::{self, Vec3},
};

pub fn make_sponza_scene(
    rendering_context: &ApplicationRenderingContext,
    framebuffer: &Framebuffer,
) -> Result<(SceneContext, ShaderContext), String> {
    let (scene_context, shader_context) = make_empty_scene(framebuffer.width_over_height)?;

    {
        let resources = (*scene_context.resources).borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Sponza meshes and materials

        let obj_result = {
            let mut material_arena = resources.material.borrow_mut();
            let mut texture_arena = resources.texture_u8.borrow_mut();

            mesh::obj::load::load_obj(
                "./examples/sponza/assets/sponza.obj",
                &mut material_arena,
                &mut texture_arena,
            )
        };

        let _atrium_geometry = obj_result.0;
        let atrium_meshes = obj_result.1;

        {
            let mut material_arena = resources.material.borrow_mut();
            let mut texture_arena = resources.texture_u8.borrow_mut();

            for entry in material_arena.entries.iter_mut().flatten() {
                let material = &mut entry.item;

                material.roughness = 1.0;
                material.metallic = 0.0;
                material.metallic_map = material.specular_exponent_map;

                material.load_all_maps(&mut texture_arena, rendering_context)?;
            }
        }

        // Assign the meshes to entities

        for mesh in atrium_meshes {
            let material_handle = mesh.material;

            let mut mesh_arena = resources.mesh.borrow_mut();

            let mesh_handle = mesh_arena.insert(Uuid::new_v4(), mesh.to_owned());

            let mut entity_arena = resources.entity.borrow_mut();

            let entity_handle =
                entity_arena.insert(Uuid::new_v4(), Entity::new(mesh_handle, material_handle));

            scene.root.add_child(SceneNode::new(
                SceneNodeType::Entity,
                Default::default(),
                Some(entity_handle),
            ))?;
        }

        // Adjust our scene's default camera.

        if let Some(camera_handle) = scene
            .root
            .find(&mut |node| *node.get_type() == SceneNodeType::Camera)
            .unwrap()
        {
            let mut camera_arena = resources.camera.borrow_mut();

            if let Ok(entry) = camera_arena.borrow_mut().get_mut(&camera_handle) {
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

            let point_light_handle = resources
                .point_light
                .borrow_mut()
                .insert(Uuid::new_v4(), light);

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
    }

    Ok((scene_context, shader_context))
}
