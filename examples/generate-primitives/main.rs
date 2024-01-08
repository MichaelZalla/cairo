extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    image::TextureMap,
    material::{cache::MaterialCache, Material},
    mesh,
    scene::Scene,
    vec::vec3::Vec3,
};

mod generate_primitives_scene;

use self::generate_primitives_scene::GeneratePrimitivesScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/generate-primitives", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(32.0, 32.0, 1, 1);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);
    let mut cone_mesh = mesh::primitive::cone::generate(2.0, 2.0, 40);
    let mut cylinder_mesh = mesh::primitive::cylinder::generate(2.0, 2.0, 40);

    // Create a new textured material
    let mut checkerboard_mat = Material::new("checkerboard".to_string());

    let mut checkerboard_texture =
        TextureMap::new(&"./examples/generate-primitives/assets/checkerboard.png");

    checkerboard_texture.load(rendering_context)?;

    checkerboard_mat.diffuse_map = Some(checkerboard_texture);

    // Point light "material"

    let mut point_light_mat = Material::new("white".to_string());
    point_light_mat.diffuse_color = color::WHITE.to_vec3() / 255.0;

    // Collect materials

    let mut material_cache = MaterialCache::new();

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_mat.name.clone());
    cube_mesh.material_name = Some(checkerboard_mat.name.clone());
    cone_mesh.material_name = Some(checkerboard_mat.name.clone());
    cylinder_mesh.material_name = Some(checkerboard_mat.name.clone());

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    plane_entity.position.x -= 5.0;
    plane_entity.position.z -= 5.0;

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.x -= 4.0;
    cube_entity.position.y += 1.0;

    let mut cone_entity = Entity::new(&cone_mesh);
    cone_entity.position.x -= 0.0;
    cone_entity.position.y += 1.0;

    let mut cylinder_entity = Entity::new(&cylinder_mesh);
    cylinder_entity.position.x += 4.0;
    cylinder_entity.position.y += 1.0;

    // let mut point_light_material = Material::new("white".to_string());
    // point_light_material.diffuse_color = color::WHITE.to_vec3() / 255.0;

    let mut point_light_mesh = mesh::primitive::cube::generate(0.2, 0.2, 0.2);

    point_light_mesh.object_name = "point_light".to_string();
    point_light_mesh.material_name = Some(point_light_mat.name.clone());

    material_cache.insert(checkerboard_mat.name.to_string(), checkerboard_mat);
    material_cache.insert(point_light_mat.name.to_string(), point_light_mat);

    let mut point_light_entity = Entity::new(&point_light_mesh);

    point_light_entity.position = Vec3 {
        x: 4.0,
        y: 3.0,
        z: 4.0,
    };

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![
        &mut plane_entity,
        &mut cube_entity,
        &mut cone_entity,
        &mut cylinder_entity,
        &mut point_light_entity,
    ];

    let entities_rwl = RwLock::new(entities);

    // Instantiate our textured cube scene
    let scene = RefCell::new(GeneratePrimitivesScene::new(
        app.canvas_width,
        app.canvas_height,
        &entities_rwl,
        &material_cache,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      seconds_since_last_update: f32|
     -> () {
        // Delegate the update to our textured cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            seconds_since_last_update,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our textured cube scene

        scene.borrow_mut().render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
