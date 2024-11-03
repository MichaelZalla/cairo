use core::f32;
use std::{f32::consts::PI, rc::Rc};

use cairo::{
    app::context::ApplicationRenderingContext,
    color::{self, Color},
    entity::Entity,
    material::Material,
    matrix::Mat4,
    mesh::{
        primitive::{cube, plane},
        Mesh,
    },
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, attenuation::LIGHT_ATTENUATION_RANGE_20_UNITS,
            directional_light::DirectionalLight, point_light::PointLight,
        },
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    transform::{quaternion::Quaternion, Transform3D},
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn make_scene(
    _resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    point_light_arena: &mut Arena<PointLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    texture_u8_arena: &mut Arena<TextureMap>,
    rendering_context: &ApplicationRenderingContext,
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

                            ambient_light.intensities = vec3::ONES * 0.1;
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

                            directional_light.intensities = vec3::ONES * 0.5;

                            let rotate_x = Quaternion::new(vec3::RIGHT, -PI / 4.0);
                            let rotate_y = Quaternion::new(vec3::UP, PI);

                            directional_light.set_direction(rotate_x * rotate_y);
                        }
                    }

                    Ok(())
                }
                SceneNodeType::Camera => {
                    if let Some(handle) = node.get_handle() {
                        if let Ok(entry) = camera_arena.get_mut(handle) {
                            let camera = &mut entry.item;

                            camera.movement_speed = 10.0;
                        }
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        },
    )?;

    // Add a textured ground plane to our scene.

    let checkerboard_material_handle = {
        let checkerboard_material = {
            let mut material = Material::new("checkerboard".to_string());

            let mut albedo_map = TextureMap::new(
                "./assets/textures/checkerboard.jpg",
                TextureMapStorageFormat::Index8(0),
            );

            // Checkerboard material

            albedo_map.sampling_options.wrapping = TextureMapWrapping::Repeat;

            albedo_map.load(rendering_context)?;

            let albedo_map_handle = texture_u8_arena.insert(albedo_map);

            material.albedo_map = Some(albedo_map_handle);

            material
        };

        material_arena.insert(checkerboard_material)
    };

    let mut plane_entity_node = {
        let mesh = plane::generate(80.0, 80.0, 8, 8);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(checkerboard_material_handle));

        let entity_handle = entity_arena.insert(entity);

        let transform = Transform3D::default();

        SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle))
    };

    // Add some emissive cubes to our scene.

    let cube_mesh = cube::generate(2.0, 2.0, 2.0);

    let cube_mesh_handle = mesh_arena.insert(cube_mesh);

    static CUBE_COLORS: [&Color; 4] = [&color::RED, &color::GREEN, &color::BLUE, &color::WHITE];

    for (color_index, color) in CUBE_COLORS.iter().enumerate() {
        let transform = {
            static ROTATION_STEP: f32 = f32::consts::TAU / CUBE_COLORS.len() as f32;

            let theta = color_index as f32 * ROTATION_STEP;

            let rotate_y = Quaternion::new(vec3::UP, theta);

            let mut position = Vec4::new(
                Vec3 {
                    x: 8.0,
                    y: 4.0,
                    z: 0.0,
                },
                1.0,
            );

            position *= *rotate_y.mat();

            let mut transform = Transform3D::default();

            transform.set_translation(position.to_vec3());

            transform
        };

        let albedo = color.to_vec3() / 255.0;

        let point_light_node = {
            let mut point_light = PointLight::new();

            point_light.intensities = albedo * 4.0;

            point_light.set_attenuation(LIGHT_ATTENUATION_RANGE_20_UNITS);

            let point_light_handle = point_light_arena.insert(point_light);

            SceneNode::new(
                SceneNodeType::PointLight,
                transform,
                Some(point_light_handle),
            )
        };

        plane_entity_node.add_child(point_light_node)?;

        let emissive_material_handle = {
            let emissive_material = {
                let mut material = Material::new(format!("emissive_material_{}", color_index));

                material.albedo = albedo;

                material.emissive_color = albedo * 4.0;

                material
            };

            material_arena.insert(emissive_material)
        };

        let cube_entity_node = {
            let entity = Entity::new(cube_mesh_handle, Some(emissive_material_handle));

            let entity_handle = entity_arena.insert(entity);

            SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle))
        };

        plane_entity_node.add_child(cube_entity_node)?;
    }

    scene.root.add_child(plane_entity_node)?;

    Ok((scene, shader_context))
}
