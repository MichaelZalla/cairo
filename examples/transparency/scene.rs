use rand::rngs::ThreadRng;

use rand_distr::{Distribution, Uniform};

use cairo::{
    app::context::ApplicationRenderingContext,
    entity::Entity,
    material::Material,
    mesh::{
        primitive::{cube, plane},
        Mesh,
    },
    resource::{arena::Arena, handle::Handle},
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
    texture::map::{TextureMap, TextureMapStorageFormat},
    transform::Transform3D,
    vec::vec3::{self, Vec3},
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_scene(
    camera_arena: &mut Arena<Camera>,
    camera_aspect_ratio: f32,
    environment_arena: &mut Arena<Environment>,
    ambient_light_arena: &mut Arena<AmbientLight>,
    directional_light_arena: &mut Arena<DirectionalLight>,
    mesh_arena: &mut Arena<Mesh>,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
    point_light_arena: &mut Arena<PointLight>,
    spot_light_arena: &mut Arena<SpotLight>,
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

    // Move our default camera.

    for entry in camera_arena.entries.iter_mut().flatten() {
        let camera = &mut entry.item;

        camera.look_vector.set_position(Vec3 {
            x: 14.0,
            y: 14.0,
            z: 14.0,
        });

        camera.look_vector.set_target(Default::default());
    }

    // Add a textured ground plane to our scene.

    let checkerboard_material_handle = {
        let checkerboard_material = {
            // Checkerboard texture map

            let albedo_map = TextureMap::new(
                "./assets/textures/checkerboard.jpg",
                TextureMapStorageFormat::Index8(0),
            );

            let albedo_map_handle = texture_u8_arena.insert(albedo_map);

            // Checkerboard material

            let mut material = Material::new("checkerboard".to_string());

            material.albedo_map = Some(albedo_map_handle);

            material.load_all_maps(texture_u8_arena, rendering_context)?;

            material
        };

        material_arena.insert(checkerboard_material)
    };

    let mut plane_entity_node = {
        let mut mesh = plane::generate(32.0, 32.0, 1, 1);

        mesh.material = Some(checkerboard_material_handle);

        let mesh_handle = mesh_arena.insert(mesh);

        let entity = Entity::new(mesh_handle, Some(checkerboard_material_handle));

        let entity_handle = entity_arena.insert(entity);

        SceneNode::new(
            SceneNodeType::Entity,
            Default::default(),
            Some(entity_handle),
        )
    };

    // Generate a bunch of cubes with various semi-transparent materials.

    let cube_mesh_handle = {
        let cube_mesh = cube::generate(2.0, 2.0, 2.0);

        mesh_arena.insert(cube_mesh)
    };

    let mut rng = rand::rng();

    let uniform = Uniform::<f32>::new(0.0, 1.0).unwrap();

    for i in 0..32 {
        let cube_entity_node = make_semi_transparent_cube(
            &mut rng,
            &uniform,
            &cube_mesh_handle,
            i,
            material_arena,
            entity_arena,
        );

        plane_entity_node.add_child(cube_entity_node)?;
    }

    scene.root.add_child(plane_entity_node)?;

    // Add a point light to our scene.

    let point_light_node = {
        let mut point_light = PointLight::new();

        point_light.intensities = Vec3::ones() * 0.8;

        let point_light_handle = point_light_arena.insert(point_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: 8.0,
            y: 16.0,
            z: 8.0,
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
        let spot_light = SpotLight::new();

        let spot_light_handle = spot_light_arena.insert(spot_light);

        let mut transform = Transform3D::default();

        transform.set_translation(Vec3 {
            x: -8.0,
            y: 32.0,
            z: 8.0,
        });

        SceneNode::new(SceneNodeType::SpotLight, transform, Some(spot_light_handle))
    };

    scene.root.add_child(spot_light_node)?;

    Ok((scene, shader_context))
}

static CUBE_SPACE_EXTENT: f32 = 16.0;

static CUBE_SPACE_HALF_EXTENT: f32 = CUBE_SPACE_EXTENT / 2.0;

static CUBE_SPACE_CENTER: Vec3 = Vec3 {
    x: 0.0,
    y: CUBE_SPACE_HALF_EXTENT,
    z: 0.0,
};

fn make_semi_transparent_cube(
    rng: &mut ThreadRng,
    uniform: &Uniform<f32>,
    cube_mesh_handle: &Handle,
    index: usize,
    material_arena: &mut Arena<Material>,
    entity_arena: &mut Arena<Entity>,
) -> SceneNode {
    let albedo = Vec3::uniform(rng, uniform);

    let alpha = uniform.sample(rng);

    let transparency = 1.0 - alpha;

    let material = Material {
        name: format!("transparent_mat_{}", index).to_string(),
        albedo,
        transparency,
        ..Default::default()
    };

    let material_handle = material_arena.insert(material);

    let entity = Entity::new(*cube_mesh_handle, Some(material_handle));

    let entity_handle = entity_arena.insert(entity);

    let transform = {
        let mut transform = Transform3D::default();

        let offset = (Vec3::uniform(rng, uniform) * 2.0 - vec3::ONES) * CUBE_SPACE_HALF_EXTENT;

        let position = CUBE_SPACE_CENTER + offset;

        transform.set_translation(position);

        let scale = 0.5 + uniform.sample(rng) * 2.5;

        transform.set_scale(vec3::ONES * scale);

        transform
    };

    SceneNode::new(SceneNodeType::Entity, transform, Some(entity_handle))
}
