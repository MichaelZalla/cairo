use uuid::Uuid;

use cairo::{
    app::context::ApplicationRenderingContext,
    buffer::framebuffer::Framebuffer,
    material::Material,
    scene::{
        context::{utils::make_cube_scene, SceneContext},
        light::{PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::vec3::Vec3,
};

#[allow(unused)]
pub(crate) fn make_textured_cube_scene(
    rendering_context: &ApplicationRenderingContext,
    framebuffer: &Framebuffer,
) -> Result<(SceneContext, ShaderContext), String> {
    let (scene_context, shader_context) = make_cube_scene(framebuffer.width_over_height).unwrap();

    {
        let mut resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Customize the cube material.

        let var_name = {
            let cube_albedo_map_handle = resources.texture_u8.borrow_mut().insert(
                Uuid::new_v4(),
                TextureMap::new("./data/obj/cobblestone.png", TextureMapStorageFormat::RGB24),
            );

            Material {
                name: "cube".to_string(),
                albedo_map: Some(cube_albedo_map_handle),
                ..Default::default()
            }
        };
        let mut cube_material = var_name;

        cube_material.load_all_maps(&mut resources.texture_u8.borrow_mut(), rendering_context)?;

        let cube_material_handle = resources
            .material
            .borrow_mut()
            .insert(Uuid::new_v4(), cube_material);

        let cube_entity_handle = scene
            .root
            .find(&mut |node| *node.get_type() == SceneNodeType::Entity)
            .unwrap()
            .unwrap();

        match resources.entity.get_mut().get_mut(&cube_entity_handle) {
            Ok(entry) => {
                let cube_entity = &mut entry.item;

                cube_entity.material = Some(cube_material_handle);
            }
            _ => panic!(),
        }

        // Add a point light to our scene.

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.7;

        point_light.position = Vec3 {
            x: 0.0,
            y: 4.0,
            z: 0.0,
        };

        let point_light_handle = resources
            .point_light
            .borrow_mut()
            .insert(Uuid::new_v4(), point_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        ))?;

        // Add a spot light to our scene.

        let spot_light = SpotLight::new();

        let spot_light_handle = resources
            .spot_light
            .borrow_mut()
            .insert(Uuid::new_v4(), spot_light);

        scene.root.add_child(SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        ))?;
    }

    Ok((scene_context, shader_context))
}
