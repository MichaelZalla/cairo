use std::{f32::consts::PI, rc::Rc};

use cairo::{
    app::context::ApplicationRenderingContext,
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
        light::{ambient_light::AmbientLight, directional_light::DirectionalLight},
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat, TextureMapWrapping},
    transform::quaternion::Quaternion,
    vec::vec3::{self, Vec3},
};

#[allow(clippy::too_many_arguments)]
pub fn make_scene(
    _resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
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
        let mut mesh = plane::generate(80.0, 80.0, 8, 8);

        mesh.material = Some(checkerboard_material_handle);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(checkerboard_material_handle));

        let entity_handle = entity_arena.insert(entity);

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
        let emissive_material = {
            let mut material = Material::new("emissive".to_string());

            material.albedo_map = Some(texture_u8_arena.insert(TextureMap::new(
                "./examples/post-effects/assets/lava.png",
                TextureMapStorageFormat::RGB24,
            )));

            material.emissive_color_map = Some(texture_u8_arena.insert(TextureMap::new(
                "./examples/post-effects/assets/lava_emissive.png",
                TextureMapStorageFormat::Index8(0),
            )));

            material
                .load_all_maps(texture_u8_arena, rendering_context)
                .unwrap();

            material
        };

        material_arena.insert(emissive_material)
    };

    let cube_entity_node = {
        let mesh = cube::generate(2.0, 2.0, 2.0);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(emissive_material_handle));

        let entity_handle = entity_arena.insert(entity);

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

    Ok((scene, shader_context))
}
