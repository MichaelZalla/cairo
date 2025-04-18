use std::f32::consts::PI;

use cairo::{
    color,
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        empty::{Empty, EmptyDisplayKind},
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, attenuation, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
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
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
    empty_arena: &mut Arena<Empty>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    scene.root.visit_mut(
        cairo::scene::node::SceneNodeGlobalTraversalMethod::DepthFirst,
        None,
        &mut |_depth, _transform, node| {
            match node.get_type() {
                SceneNodeType::AmbientLight => {
                    node.get_transform_mut().set_translation(Vec3 {
                        x: -12.0,
                        y: 6.0,
                        z: 8.0,
                    });
                }
                SceneNodeType::DirectionalLight => {
                    node.get_transform_mut().set_translation(Vec3 {
                        x: -4.0,
                        y: 6.0,
                        z: 8.0,
                    });
                }
                _ => (),
            }

            Ok(())
        },
    )?;

    // Add various Empties to our scene.

    static EMPTY_DISPLAY_KINDS: [EmptyDisplayKind; 7] = [
        EmptyDisplayKind::Axes,
        EmptyDisplayKind::Arrow,
        EmptyDisplayKind::Square,
        EmptyDisplayKind::Cube,
        EmptyDisplayKind::Circle(16),
        EmptyDisplayKind::Sphere(16),
        EmptyDisplayKind::Capsule(16, 2.0),
    ];

    for (index, kind) in EMPTY_DISPLAY_KINDS.iter().enumerate() {
        let empty_node = {
            let empty = Empty(*kind);

            let empty_handle = empty_arena.insert(empty);

            let mut transform = Transform3D::default();

            static GRID_WIDTH: f32 = EMPTY_DISPLAY_KINDS.len() as f32;
            static GRID_SCALE: f32 = 4.0;

            transform.set_translation(Vec3 {
                x: (-GRID_WIDTH / 2.0 + index as f32 + 0.5) * GRID_SCALE,
                y: 2.0,
                z: -0.5,
            });

            let mut node = SceneNode::new(SceneNodeType::Empty, transform, Some(empty_handle));

            node.name
                .replace(format!("{}_empty_{}", kind, index).to_string());

            node
        };

        scene.root.add_child(empty_node)?;
    }

    // Add a point light to our scene.

    let point_light_node = {
        let mut point_light = PointLight::new();

        point_light.intensities = color::RED.to_vec3() / 255.0;

        point_light.set_attenuation(attenuation::LIGHT_ATTENUATION_RANGE_7_UNITS);

        let point_light_handle = point_light_arena.insert(point_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 4.0,
            y: 6.0,
            z: 8.0,
        });

        let mut node = SceneNode::new(
            SceneNodeType::PointLight,
            transform,
            Some(point_light_handle),
        );

        node.name.replace("point_light_1".to_string());

        node
    };

    scene.root.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut spot_light = SpotLight::new();

        spot_light.set_inner_cutoff_angle(PI / 32.0);
        spot_light.set_outer_cutoff_angle(PI / 8.0);

        spot_light.intensities = color::YELLOW.to_vec3() / 255.0 * 2.0;

        spot_light.set_attenuation(attenuation::LIGHT_ATTENUATION_RANGE_13_UNITS);

        let spot_light_handle = spot_light_arena.insert(spot_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 12.0,
            y: 6.0,
            z: 8.0,
        });

        let mut node = SceneNode::new(SceneNodeType::SpotLight, transform, Some(spot_light_handle));

        node.name.replace("spot_light_1".to_string());

        node
    };

    scene.root.add_child(spot_light_node)?;

    Ok((scene, shader_context))
}
