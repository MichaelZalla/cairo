use std::rc::Rc;

use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    geometry::accelerator::static_triangle_bvh::StaticTriangleBVH,
    material::Material,
    mesh::{
        Mesh,
        obj::load::{LoadObjResult, ProcessGeometryFlags, load_obj},
    },
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight,
            point_light::PointLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    texture::map::TextureMap,
};

#[allow(clippy::too_many_arguments)]
pub fn make_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    _point_light_arena: &mut Arena<PointLight>,
    _spot_light_arena: &mut Arena<SpotLight>,
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

    let camera_handle = scene
        .root
        .find(|node| *node.get_type() == SceneNodeType::Camera)
        .unwrap()
        .unwrap();

    if let Ok(entry) = camera_arena.get_mut(&camera_handle) {
        let camera = &mut entry.item;

        camera.movement_speed = 200.0;
    }

    // Load an OBJ model into our scene.

    let LoadObjResult(_model_geometry, model_meshes) = load_obj(
        "./data/obj/LowPoly/low_poly_game_level.obj",
        material_arena,
        texture_u8_arena,
        Some(ProcessGeometryFlags::empty() | ProcessGeometryFlags::CENTER),
    );

    for entry in material_arena.entries.iter_mut().flatten() {
        let material = &mut entry.item;

        material.roughness = 1.0;
        material.metallic = 0.0;
        material.metallic_map = material.specular_exponent_map;

        material.load_all_maps(texture_u8_arena, rendering_context)?;
    }

    // Assign the meshes to entities

    for mesh in model_meshes {
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
