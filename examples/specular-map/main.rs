extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::{cache::MaterialCache, Material},
    mesh,
    scene::Scene,
    shader::ShaderContext,
    texture::TextureMap,
    time::TimingInfo,
    vec::vec3::Vec3,
};

mod specular_map_scene;

use self::specular_map_scene::SpecularMapScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/specular-map", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

    // Initialize materials

    // Checkerboard pattern

    let mut checkerboard_material = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(&"./assets/textures/checkerboard.jpg");

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_material.specular_map = Some(checkerboard_specular_map);

    // Metal-wrapped wooden container

    let mut container_material = Material::new("container".to_string());

    let container_diffuse_map = TextureMap::new(&"./examples/specular-map/assets/container2.png");

    let container_specular_map =
        TextureMap::new(&"./examples/specular-map/assets/container2_specular.png");

    container_material.diffuse_map = Some(container_diffuse_map);

    container_material.specular_map = Some(container_specular_map);

    container_material.load_all_maps(rendering_context).unwrap();

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_material.name.clone());

    cube_mesh.material_name = Some(container_material.name.clone());

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    material_cache.insert(checkerboard_material);

    material_cache.insert(container_material);

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.y += 1.5;

    // Orbiting point light

    let mut point_light_material = Material::new("white".to_string());
    point_light_material.diffuse_color = color::WHITE.to_vec3() / 255.0;

    let mut point_light_mesh = mesh::primitive::cube::generate(0.2, 0.2, 0.2);

    point_light_mesh.object_name = "point_light".to_string();
    point_light_mesh.material_name = Some(point_light_material.name.clone());

    let mut point_light_entity = Entity::new(&point_light_mesh);

    point_light_entity.position = Vec3 {
        x: 0.0,
        y: 5.0,
        z: 0.0,
    };

    material_cache.insert(point_light_material);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> =
        vec![&mut plane_entity, &mut cube_entity, &mut point_light_entity];

    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our textured cube scene
    let scene = RefCell::new(SpecularMapScene::new(
        app.canvas_width,
        app.canvas_height,
        &entities_rwl,
        &material_cache,
        &shader_context_rwl,
    ));

    // Set up our app
    let mut update = |timing_info: &TimingInfo,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
        // Delegate the update to our textured cube scene

        scene.borrow_mut().update(
            &timing_info,
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our textured cube scene

        scene.borrow_mut().render();

        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
