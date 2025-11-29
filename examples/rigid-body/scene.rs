use cairo::{
    entity::Entity,
    material::Material,
    mesh::primitive::sphere,
    physics::simulation::rigid_body::RigidBodyKind,
    scene::{
        context::{SceneContext, utils::make_empty_scene},
        graph::SceneGraph,
        node::{SceneNode, SceneNodeType},
    },
    shader::context::ShaderContext,
    transform::Transform3D,
    vec::vec3,
};

use crate::simulation::Simulation;

pub fn make_scene(
    camera_aspect_ratio: f32,
    scene_context: &SceneContext,
    simulation: &Simulation,
    ground_mesh: Option<&cairo::mesh::Mesh>,
) -> Result<(SceneGraph, ShaderContext), String> {
    let resources = &scene_context.resources;

    let mut camera_arena = resources.camera.borrow_mut();
    let mut environment_arena = resources.environment.borrow_mut();
    let mut ambient_light_arena = resources.ambient_light.borrow_mut();
    let mut directional_light_arena = resources.directional_light.borrow_mut();

    let (mut scene, shader_context) = make_empty_scene(
        &mut camera_arena,
        camera_aspect_ratio,
        &mut environment_arena,
        &mut ambient_light_arena,
        &mut directional_light_arena,
    )?;

    let active_camera_entry = camera_arena
        .entries
        .iter_mut()
        .flatten()
        .find(|e| e.item.is_active)
        .unwrap();

    let camera = &mut active_camera_entry.item;

    camera.set_projection_z_near(0.1);
    camera.set_projection_z_far(1_000.0);

    // Enable shadow mapping for the directional light
    let directional_light_entry = directional_light_arena
        .entries
        .iter_mut()
        .flatten()
        .next()
        .unwrap();

    let directional_light = &mut directional_light_entry.item;

    directional_light.intensities = vec3::ONES * 0.6;

    // directional_light.enable_shadow_maps(1024, 48.0, resources.clone());

    // Generate shared sphere mesh
    let radius = match simulation.rigid_bodies[0].kind {
        RigidBodyKind::Circle(_) => panic!(),
        RigidBodyKind::Sphere(radius) => radius,
    };

    let sphere_mesh = sphere::generate(1.0, 12, 12);

    // Create sphere entities for each rigid body
    let mut mesh_arena = resources.mesh.borrow_mut();
    let mut material_arena = resources.material.borrow_mut();
    let mut entity_arena = resources.entity.borrow_mut();

    // Insert shared sphere mesh once
    let shared_sphere_mesh_handle = mesh_arena.insert(sphere_mesh);

    // Insert ground mesh if provided
    let ground_mesh_handle = ground_mesh.map(|mesh| mesh_arena.insert(mesh.clone()));

    for (index, rigid_body) in simulation.rigid_bodies.iter().enumerate() {
        let mut sphere_material = Material::new(format!("sphere_material_{}", index));

        // Set material albedo to match rigid body color (convert from 0-255 to 0-1 range)
        sphere_material.albedo = vec3::Vec3 {
            x: rigid_body.color.r / 255.0,
            y: rigid_body.color.g / 255.0,
            z: rigid_body.color.b / 255.0,
        };

        let sphere_material_handle = material_arena.insert(sphere_material);

        let sphere_entity = Entity::new(shared_sphere_mesh_handle, Some(sphere_material_handle));
        let sphere_entity_handle = entity_arena.insert(sphere_entity);

        let mut transform = Transform3D::default();
        transform.set_scale(vec3::ONES * radius);
        transform.set_translation(*rigid_body.transform.translation());

        let mut sphere_node =
            SceneNode::new(SceneNodeType::Entity, transform, Some(sphere_entity_handle));

        sphere_node.name.replace(format!("sphere_{}", index));

        scene.root.add_child(sphere_node)?;
    }

    // Add ground plane entity if ground mesh was provided
    if let Some(mesh_handle) = ground_mesh_handle {
        let ground_material = Material::new("ground_material".to_string());
        let ground_material_handle = material_arena.insert(ground_material);

        let ground_entity = Entity::new(mesh_handle, Some(ground_material_handle));
        let ground_entity_handle = entity_arena.insert(ground_entity);

        let ground_node = SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(ground_entity_handle),
        );

        scene.root.add_child(ground_node)?;
    }

    Ok((scene, shader_context))
}
