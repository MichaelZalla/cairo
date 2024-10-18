use cairo::{
    scene::{
        context::{utils::make_cube_scene, SceneContext},
        light::{PointLight, SpotLight},
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    vec::vec3::Vec3,
};

pub fn make_spinning_cube_scene(
    aspect_ratio: f32,
) -> Result<(SceneContext, ShaderContext), String> {
    let (scene_context, shader_context) = make_cube_scene(aspect_ratio)?;

    {
        let resources = scene_context.resources.borrow_mut();
        let scene = &mut scene_context.scenes.borrow_mut()[0];

        // Add a point light to the scene.

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.4;

        let point_light_handle = resources.point_light.borrow_mut().insert(point_light);

        let mut point_light_node = SceneNode::new(
            SceneNodeType::PointLight,
            Default::default(),
            Some(point_light_handle),
        );

        point_light_node.get_transform_mut().set_translation(Vec3 {
            x: 0.0,
            y: 5.0,
            z: 0.0,
        });

        scene.root.add_child(point_light_node)?;

        // Add a spot light to the scene.

        let spot_light = SpotLight::new();

        let spot_light_handle = resources.spot_light.borrow_mut().insert(spot_light);

        let mut spot_light_node = SceneNode::new(
            SceneNodeType::SpotLight,
            Default::default(),
            Some(spot_light_handle),
        );

        spot_light_node.get_transform_mut().set_translation(Vec3 {
            x: 0.0,
            y: 5.0,
            z: 0.0,
        });

        scene.root.add_child(spot_light_node)?;
    }

    Ok((scene_context, shader_context))
}
