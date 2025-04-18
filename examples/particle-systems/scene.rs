use std::{
    f32::consts::{PI, TAU},
    rc::Rc,
};

use cairo::{
    entity::Entity,
    geometry::accelerator::static_triangle_bvh::StaticTriangleBVH,
    material::Material,
    mesh::{
        obj::load::{load_obj, LoadObjResult, ProcessGeometryFlag},
        Mesh,
    },
    random::sampler::{RandomSampler, RangeSampler},
    resource::arena::Arena,
    scene::{
        camera::Camera,
        context::utils::make_empty_scene,
        environment::Environment,
        graph::SceneGraph,
        light::{
            ambient_light::AmbientLight, directional_light::DirectionalLight, spot_light::SpotLight,
        },
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    transform::{quaternion::Quaternion, Transform3D},
    vec::vec3::{self, Vec3},
};

#[allow(clippy::too_many_arguments)]
pub fn make_particles_scene(
    resources: &Rc<SceneResources>,
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
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

    // Spot light grid.

    static GRID_SIZE: f32 = 25.0;
    static GRID_SIZE_OVER_2: f32 = GRID_SIZE * 0.5;

    static ROWS: usize = 1;
    static ROWS_ALPHA_STEP: f32 = 1.0 / ROWS as f32;

    static COLUMNS: usize = 1;
    static COLUMNS_ALPHA_STEP: f32 = 1.0 / COLUMNS as f32;

    let (mut u, mut v);

    for x in 0..COLUMNS + 1 {
        u = x as f32 * COLUMNS_ALPHA_STEP;

        for z in 0..ROWS + 1 {
            v = z as f32 * ROWS_ALPHA_STEP;

            let position = Vec3 {
                x: -GRID_SIZE_OVER_2 + u * GRID_SIZE,
                z: -GRID_SIZE_OVER_2 + v * GRID_SIZE,
                y: 40.0,
            };

            let mut transform = Transform3D::default();

            transform.set_translation(position);

            let spot_light_node = {
                let mut spot_light = SpotLight::new();

                spot_light.look_vector.set_position(position);
                spot_light.look_vector.set_target(position - vec3::UP);

                let spot_light_handle = spot_light_arena.insert(spot_light);

                SceneNode::new(SceneNodeType::SpotLight, transform, Some(spot_light_handle))
            };

            scene.root.add_child(spot_light_node)?;
        }
    }

    // Static level geometry.

    let mut level_mesh = {
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        let LoadObjResult(_level_geometry, level_meshes) = load_obj(
            "./data/blender/collision-level/collision-level_005.obj",
            material_arena,
            &mut texture_u8_arena,
            Some(ProcessGeometryFlag::Null | ProcessGeometryFlag::Center),
        );

        level_meshes.last().unwrap().to_owned()
    };

    let bvh = StaticTriangleBVH::new(&level_mesh);

    level_mesh.collider = Some(Rc::new(bvh));

    // Assign the level mesh to an entity.

    let level_entity_node = {
        let material_handle = level_mesh.material;

        let mesh_handle = mesh_arena.insert(level_mesh);

        let entity_handle = entity_arena.insert(Entity::new(mesh_handle, material_handle));

        SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        )
    };

    let mut sampler: RandomSampler<256> = Default::default();

    // Seed the simulation's random number sampler.

    sampler.seed().unwrap();

    for _ in 0..2 {
        let mut node = level_entity_node.clone();

        let transform = node.get_transform_mut();

        let random_scale = vec3::ONES * sampler.sample_range_normal(1.0, 0.25);

        let random_rotation = Quaternion::new(vec3::UP, sampler.sample_range_uniform(0.0, TAU))
            * Quaternion::new(vec3::FORWARD, sampler.sample_range_uniform(0.0, PI / 8.0))
            * Quaternion::new(vec3::RIGHT, sampler.sample_range_uniform(0.0, PI / 8.0));

        let random_translation = Vec3 {
            x: sampler.sample_range_uniform(-12.0, 12.0),
            y: sampler.sample_range_uniform(0.0, 10.0),
            z: sampler.sample_range_uniform(-12.0, 12.0),
        };

        transform.set_scale(random_scale);
        transform.set_rotation(random_rotation);
        transform.set_translation(random_translation);

        scene.root.add_child(node)?;
    }

    Ok((scene, shader_context))
}
