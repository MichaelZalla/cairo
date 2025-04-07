use std::{path::Path, rc::Rc};

use cairo::{
    app::context::ApplicationRenderingContext,
    color::Color,
    entity::Entity,
    geometry::accelerator::static_triangle_bvh::StaticTriangleBVH,
    material::Material,
    matrix::Mat4,
    mesh::{
        obj::load::{load_obj, LoadObjResult, ProcessGeometryFlag},
        Mesh,
    },
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, attenuation::LIGHT_ATTENUATION_RANGE_600_UNITS,
            directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
        resources::SceneResources,
        skybox::Skybox,
    },
    shader::context::ShaderContext,
    texture::{
        cubemap::CubeMap,
        map::{TextureMap, TextureMapStorageFormat},
    },
    transform::Transform3D,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

#[allow(clippy::too_many_arguments)]
pub fn make_sponza_scene(
    resources: &Rc<SceneResources>,
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
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
    skybox_arena: &mut Arena<Skybox>,
    texture_vec2_arena: &mut Arena<TextureMap<Vec2>>,
    cubemap_vec3_arena: &mut Arena<CubeMap<Vec3>>,
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

                            ambient_light.intensities = vec3::ONES * 0.005;
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

                            directional_light.intensities = vec3::ONES * 0.005;

                            directional_light.enable_shadow_maps(384, 5_000.0, resources.clone());
                        }
                    }

                    Ok(())
                }
                SceneNodeType::Camera => {
                    if let Some(handle) = node.get_handle() {
                        if let Ok(entry) = camera_arena.get_mut(handle) {
                            let camera = &mut entry.item;

                            camera.look_vector.set_position(Vec3 {
                                x: 1204.75,
                                y: (-59.02 + 87.70) / 2.0,
                                z: 85.99,
                            });

                            camera
                                .look_vector
                                .set_target(camera.look_vector.get_position() + vec3::RIGHT * -1.0);

                            camera.movement_speed = 300.0;

                            camera.set_projection_z_far(10_000.0);
                        }
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        },
    )?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut light = PointLight::new();

        light.intensities = Color::rgb(255, 85, 0).to_vec3() / 255.0 * 5.0;

        light.set_attenuation(LIGHT_ATTENUATION_RANGE_600_UNITS);

        light.enable_shadow_maps(192, 5_000.0, resources.clone());

        let point_light_handle = point_light_arena.insert(light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 0.0,
            y: 200.00,
            z: -75.0,
        });

        SceneNode::new(
            SceneNodeType::PointLight,
            transform,
            Some(point_light_handle),
        )
    };

    scene.root.add_child(point_light_node)?;

    // Add a spot light to our scene.

    let spot_light_node = {
        let mut light = SpotLight::new();

        light.look_vector.set_target(
            light.look_vector.get_position()
                + Vec3 {
                    x: 0.001,
                    y: -1.0,
                    z: 0.001,
                },
        );

        light.intensities = vec3::ONES * 5.0;

        light.set_attenuation(LIGHT_ATTENUATION_RANGE_600_UNITS);

        let light_handle = spot_light_arena.insert(light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 0.0,
            y: 300.0,
            z: 0.0,
        });

        SceneNode::new(SceneNodeType::SpotLight, transform, Some(light_handle))
    };

    scene.root.add_child(spot_light_node)?;

    // Add a skybox to our scene.

    let skybox_node = {
        let mut skybox_cubemap: CubeMap = CubeMap::cross(
            "examples/skybox/assets/cross/skybox_texture.jpg",
            TextureMapStorageFormat::RGB24,
        );

        skybox_cubemap.load(rendering_context)?;

        let mut skybox = Skybox {
            is_hdr: true,
            ..Default::default()
        };

        let hdr_path = Path::new("./examples/sponza/assets/fireplace_1k.hdr");

        skybox.load_hdr(texture_vec2_arena, cubemap_vec3_arena, hdr_path);

        let skybox_handle = skybox_arena.insert(skybox);

        SceneNode::new(
            SceneNodeType::Skybox,
            Default::default(),
            Some(skybox_handle),
        )
    };

    for node in scene.root.children_mut().as_mut().unwrap() {
        if *node.get_type() == SceneNodeType::Environment {
            node.add_child(skybox_node)?;

            break;
        }
    }

    // Sponza meshes and materials

    let LoadObjResult(_atrium_geometry, atrium_meshes) = load_obj(
        "./examples/sponza/assets/sponza.obj",
        material_arena,
        texture_u8_arena,
        Some(ProcessGeometryFlag::Null | ProcessGeometryFlag::Center),
    );

    for entry in material_arena.entries.iter_mut().flatten() {
        let material = &mut entry.item;

        material.roughness = 1.0;
        material.metallic = 0.0;
        material.metallic_map = material.specular_exponent_map;

        material.load_all_maps(texture_u8_arena, rendering_context)?;
    }

    // Assign the meshes to entities

    for mesh in atrium_meshes {
        let mut owned_mesh = mesh.to_owned();

        if owned_mesh.faces.is_empty() {
            continue;
        }

        let bvh = StaticTriangleBVH::new(&owned_mesh);

        owned_mesh.collider.replace(Rc::new(bvh));

        let material_handle = owned_mesh.material;

        let mesh_handle = mesh_arena.insert(owned_mesh);

        let entity_handle = entity_arena.insert(Entity::new(mesh_handle, material_handle));

        scene.root.add_child(SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        ))?;
    }

    Ok((scene, shader_context))
}
