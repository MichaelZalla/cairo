use std::{f32::consts::PI, path::Path, rc::Rc};

use cairo::{
    color,
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::{
        primitive::{cone, cube, cylinder},
        Mesh,
    },
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, attenuation::LightAttenuation,
            directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
        resources::SceneResources,
        skybox::Skybox,
    },
    shader::context::ShaderContext,
    texture::{cubemap::CubeMap, map::TextureMap},
    transform::{quaternion::Quaternion, Transform3D},
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_scene(
    resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    skybox_arena: &mut Arena<Skybox>,
    texture_vec2_arena: &mut Arena<TextureMap<Vec2>>,
    cubemap_vec3_arena: &mut Arena<CubeMap<Vec3>>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let (mut scene, shader_context) = make_empty_scene(
        camera_arena,
        camera_aspect_ratio,
        environment_arena,
        ambient_light_arena,
        directional_light_arena,
    )?;

    scene.root.visit_mut(
        SceneNodeGlobalTraversalMethod::DepthFirst,
        None,
        &mut |_current_depth: usize, _current_world_transform: Mat4, node: &mut SceneNode| {
            match node.get_type() {
                SceneNodeType::AmbientLight => {
                    if let Some(handle) = node.get_handle() {
                        if let Ok(entry) = ambient_light_arena.get_mut(handle) {
                            let ambient_light = &mut entry.item;

                            ambient_light.intensities = Vec3 {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            } * 0.15;
                        }
                    }

                    Ok(())
                }
                SceneNodeType::DirectionalLight => {
                    let transform = node.get_transform_mut();

                    transform.set_translation(Vec3 {
                        x: 0.0,
                        y: 15.0,
                        z: 0.0,
                    });

                    if let Some(handle) = node.get_handle() {
                        if let Ok(entry) = directional_light_arena.get_mut(handle) {
                            let directional_light = &mut entry.item;

                            directional_light.intensities = color::ORANGE.to_vec3() / 255.0 * 0.15;

                            let rotate_x = Quaternion::new(vec3::RIGHT, -PI / 4.0);
                            let rotate_y = Quaternion::new(vec3::UP, PI);

                            directional_light.set_direction(rotate_x * rotate_y);

                            directional_light.enable_shadow_maps(384, 100.0, resources.clone());
                        }
                    }

                    Ok(())
                }
                SceneNodeType::Camera => {
                    if let Some(handle) = node.get_handle() {
                        if let Ok(entry) = camera_arena.get_mut(handle) {
                            let camera = &mut entry.item;

                            camera.look_vector.set_position(Vec3 {
                                x: 0.0,
                                y: 16.0,
                                z: -50.0,
                            });

                            camera.look_vector.set_target_position(Vec3 {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            });

                            camera.set_projection_z_far(150.0);
                        }
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        },
    )?;

    // Add point lights to our scene.

    for (index, color) in [color::RED].iter().enumerate() {
        let point_light_node = {
            let point_light = {
                let mut light = PointLight::new();

                light.intensities = color.to_vec3() / 255.0 * 5.0;

                light.attenuation = LightAttenuation::new(1.0, 0.09, 0.032);

                light.influence_distance = light.attenuation.get_approximate_influence_distance();

                light.enable_shadow_maps(192, 250.0, resources.clone());

                light
            };

            let point_light_handle = point_light_arena.insert(point_light);

            let mut transform = Transform3D::default();

            let y = 12.0 + index as f32 * 2.0;

            let factor = (y - 5.0) / 2.0;

            transform.set_translation(Vec3 {
                x: 10.0 * (PI / 2.0 * factor).sin(),
                y,
                z: 10.0 * (PI / 2.0 * factor).cos(),
            });

            SceneNode::new(
                SceneNodeType::PointLight,
                transform,
                Some(point_light_handle),
            )
        };

        scene.root.add_child(point_light_node).unwrap();
    }

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut spot_light = SpotLight::new();

        spot_light.intensities = Vec3 {
            x: 1.0,
            y: 1.0,
            z: 0.0,
        } * 2.0;

        let spot_light_handle = spot_light_arena.insert(spot_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 25.0,
            y: 25.0,
            z: 0.0,
        });

        SceneNode::new(SceneNodeType::SpotLight, transform, Some(spot_light_handle))
    };

    scene.root.add_child(spot_light_node)?;

    // Add a skybox to our scene.

    for node in scene.root.children_mut().as_mut().unwrap() {
        if node.is_type(SceneNodeType::Environment) {
            let skybox_node = {
                let mut skybox = Skybox {
                    is_hdr: true,
                    ..Default::default()
                };

                let hdr_path = Path::new("./examples/ibl/assets/kloppenheim_06_puresky_4k.hdr");

                skybox.load_hdr(texture_vec2_arena, cubemap_vec3_arena, hdr_path);

                let skybox_handle = skybox_arena.insert(skybox);

                SceneNode::new(
                    SceneNodeType::Skybox,
                    Default::default(),
                    Some(skybox_handle),
                )
            };

            node.add_child(skybox_node)?;
        }
    }

    // Create a rough material.

    let material_handle = {
        let mut material = Material::new("mat".to_string());

        material.albedo = vec3::ONES;
        material.roughness = 0.7;
        material.metallic = 0.2;

        material_arena.insert(material)
    };

    // Add a ground plane to our scene.

    let plane_entity_node = {
        let entity = {
            let mut mesh = cube::generate(200.0, 1.0, 200.0);

            mesh.object_name = Some("plane".to_string());

            let mesh_handle = mesh_arena.insert(mesh);

            Entity::new(mesh_handle, Some(material_handle))
        };

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            z: 3.0,
            ..Default::default()
        });

        node
    };

    scene.root.add_child(plane_entity_node)?;

    // Add a cube entity.

    let cube_entity_node = {
        let mesh = cube::generate(5.0, 5.0, 5.0);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            x: -16.0,
            y: 5.0,
            ..Default::default()
        });

        node
    };

    scene.root.add_child(cube_entity_node)?;

    // Add a cone entity.

    let cone_entity_node = {
        let mesh = cone::generate(2.5, 5.0, 16);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            x: 0.0,
            y: 5.0,
            ..Default::default()
        });

        node
    };

    scene.root.add_child(cone_entity_node)?;

    // Add a cylinder entity.

    let cylinder_entity_node = {
        let mesh = cylinder::generate(2.5, 5.0, 16);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(material_handle));

        let entity_handle = entity_arena.insert(entity);

        let mut node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        );

        node.get_transform_mut().set_translation(Vec3 {
            x: 16.0,
            y: 5.0,
            ..Default::default()
        });

        node
    };

    scene.root.add_child(cylinder_entity_node)?;

    Ok((scene, shader_context))
}