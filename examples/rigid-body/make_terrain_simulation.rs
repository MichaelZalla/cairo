use std::rc::Rc;

use cairo::{
    color::Color,
    geometry::{
        accelerator::{
            hash_grid::{HashGrid, HashGridInsertionStrategy},
            static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
            static_triangle_tlas::StaticTriangleTLAS,
        },
        primitives::aabb::AABB,
    },
    matrix::Mat4,
    mesh::{
        Mesh,
        mesh_geometry::MeshGeometry,
        obj::load::{LoadObjResult, ProcessGeometryFlags, load_obj},
        primitive::plane,
    },
    physics::simulation::{
        collision::colliders::plane_collider::PlaneCollider,
        force::gravity::GRAVITY_RIGID_BODY_FORCE,
        rigid_body::{
            RigidBody, RigidBodyDebugFlags, RigidBodyKind,
            rigid_body_simulation_state::DynRigidBodyForce,
        },
    },
    random::sampler::{DirectionSampler, RandomSampler, RangeSampler},
    scene::context::SceneContext,
    vec::vec3::{self, Vec3},
};

use crate::simulation::Simulation;

/// Creates a wavy (sine wave) terrain mesh.
pub fn make_sine_wave_terrain(
    width: f32,
    depth: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
    amplitude: f32,
    frequency: f32,
) -> Mesh {
    // Generate base plane mesh
    let base_mesh = plane::generate(width, depth, subdivisions_x, subdivisions_z);
    let old_geometry = &base_mesh.geometry;

    // Clone and modify vertices with sine wave displacement
    let mut new_vertices = old_geometry.vertices.to_vec();

    for vertex in new_vertices.iter_mut() {
        // Sine wave in both X and Z directions
        let wave_x = (vertex.x * frequency).sin();
        let wave_z = (vertex.z * frequency).cos();

        // Combine waves and apply amplitude
        vertex.y += (wave_x + wave_z) * amplitude;
    }

    // Create new geometry with modified vertices
    let new_geometry = MeshGeometry {
        vertices: new_vertices.into_boxed_slice(),
        normals: old_geometry.normals.clone(),
        uvs: old_geometry.uvs.clone(),
    };

    // Create new mesh with updated geometry
    let mut new_mesh = Mesh {
        object_source: base_mesh.object_source.clone(),
        object_name: base_mesh.object_name.clone(),
        group_name: base_mesh.group_name.clone(),
        material_source: base_mesh.material_source.clone(),
        material: base_mesh.material,
        geometry: Rc::new(new_geometry),
        faces: base_mesh.faces.clone(),
        aabb: base_mesh.aabb, // Temporary, will be recomputed
        collider: None,       // Will be rebuilt
    };

    // Recompute AABB based on new vertices
    new_mesh.aabb = AABB::from(&new_mesh);

    new_mesh
}

/// Creates a simulation with rigid body spheres placed randomly over a terrain mesh.
///
/// The function automatically determines spawn bounds based on the mesh's AABB,
/// placing spheres within the horizontal (X, Z) bounds and above the mesh.
pub fn make_simulation_from_mesh(
    ground_mesh: &Mesh,
    num_spheres: usize,
    sphere_radius: f32,
    sphere_mass: f32,
    initial_velocity_probability: f32,
    sampler: &mut RandomSampler<1024>,
) -> Simulation {
    // Forces
    let forces: Vec<Box<DynRigidBodyForce>> = vec![Box::new(GRAVITY_RIGID_BODY_FORCE)];

    // Determine spawn bounds from mesh AABB
    let mesh_aabb = ground_mesh.aabb;
    let mesh_min = mesh_aabb.min;
    let mesh_max = mesh_aabb.max;

    // Use 80% of the mesh horizontal extent to keep spheres away from edges
    let margin = 0.1;
    let x_min = mesh_min.x + (mesh_max.x - mesh_min.x) * margin;
    let x_max = mesh_max.x - (mesh_max.x - mesh_min.x) * margin;
    let z_min = mesh_min.z + (mesh_max.z - mesh_min.z) * margin;
    let z_max = mesh_max.z - (mesh_max.z - mesh_min.z) * margin;

    // Spawn height: start well above the highest point of the mesh
    let y_base = mesh_max.y + 10.0;
    let y_variation = 5.0;

    let mut spheres = Vec::new();

    // Create randomly placed spheres
    for _ in 0..num_spheres {
        let x = sampler.sample_range_uniform(x_min, x_max);
        let z = sampler.sample_range_uniform(z_min, z_max);
        let y = y_base + sampler.sample_range_uniform(0.0, y_variation);

        let center = Vec3 { x, y, z };

        let mut sphere = RigidBody::new(RigidBodyKind::Sphere(sphere_radius), sphere_mass, center);

        // Random colors (0-255 range)
        sphere.color = Color {
            r: sampler.sample_range_uniform(100.0, 255.0),
            g: sampler.sample_range_uniform(100.0, 255.0),
            b: sampler.sample_range_uniform(100.0, 255.0),
            a: 255.0,
        };

        // Some spheres get initial velocity and spin
        if sampler.sample_range_uniform(0.0, 1.0) < initial_velocity_probability {
            // Random horizontal direction
            let direction = sampler.sample_direction_uniform();

            let horizontal_direction = Vec3 {
                y: 0.0,
                ..direction
            }
            .as_normal();

            // Random speed
            let speed = sampler.sample_range_uniform(1.0, 4.0);

            sphere.linear_momentum = horizontal_direction * speed * sphere_mass;

            // Random spin axis and rate
            let spin_axis = sampler.sample_direction_uniform();
            let spin_rate = sampler.sample_range_uniform(0.5, 3.0);

            sphere.angular_momentum = spin_axis * spin_rate;
        }

        sphere.debug_flags |= RigidBodyDebugFlags::DRAW_LINEAR_VELOCITY
            | RigidBodyDebugFlags::DRAW_ANGULAR_VELOCITY
            | RigidBodyDebugFlags::DRAW_STATIC_CONTACTS;

        spheres.push(sphere);
    }

    // Static plane colliders - safety net below terrain
    let safety_net_y = mesh_min.y - 20.0;

    let static_plane_colliders = vec![PlaneCollider::new(
        Vec3 {
            x: 0.0,
            y: safety_net_y,
            z: 0.0,
        },
        vec3::UP,
    )];

    // Build BVH for the terrain
    let ground_bvh = Rc::new(StaticTriangleBVH::new(ground_mesh));

    // Create TLAS with the terrain mesh
    let identity = Mat4::identity();
    let ground_bvh_instance = StaticTriangleBVHInstance::new(&ground_bvh, identity, identity);
    let ground_tlas = StaticTriangleTLAS::new(vec![ground_bvh_instance]);

    Simulation {
        forces,
        rigid_bodies: spheres,
        static_plane_colliders,
        ground_tlas,
        hash_grid: HashGrid::new(HashGridInsertionStrategy::default(), sphere_radius * 2.0),
        visualize_hash_grid: false,
        visualize_bvh: false,
        tick_count: 0,
    }
}

/// Creates a terrain simulation with sine wave displaced ground mesh.
pub fn make_terrain_simulation(
    sampler: &mut RandomSampler<1024>,
    _scene_context: &SceneContext,
) -> (Simulation, Option<Mesh>) {
    // Create sine wave terrain mesh
    let ground_mesh = make_sine_wave_terrain(
        100.0, // width
        100.0, // depth
        30,    // subdivisions_x
        30,    // subdivisions_z
        3.0,   // amplitude
        0.15,  // frequency
    );

    // Create simulation with 100 spheres
    let simulation = make_simulation_from_mesh(
        &ground_mesh,
        100, // num_spheres
        1.0, // sphere_radius
        1.0, // sphere_mass
        0.4, // initial_velocity_probability (40% get velocity)
        sampler,
    );

    (simulation, Some(ground_mesh))
}

/// Creates a terrain simulation from an OBJ file.
///
/// Loads a mesh from the specified OBJ file and uses it as collision geometry.
/// The mesh is centered during loading.
pub fn make_obj_terrain_simulation(
    sampler: &mut RandomSampler<1024>,
    scene_context: &SceneContext,
) -> (Simulation, Option<Mesh>) {
    let resources = &scene_context.resources;

    // Load mesh from OBJ file
    let level_mesh = {
        let mut material_arena = resources.material.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        let LoadObjResult(_level_geometry, level_meshes) = load_obj(
            "./data/blender/collision-level/collision-level_005.obj",
            &mut material_arena,
            &mut texture_u8_arena,
            Some(ProcessGeometryFlags::empty() | ProcessGeometryFlags::CENTER),
        );

        level_meshes.last().unwrap().to_owned()
    };

    // Create simulation with 100 spheres
    let simulation = make_simulation_from_mesh(&level_mesh, 100, 1.0, 1.0, 0.4, sampler);

    (simulation, Some(level_mesh))
}
