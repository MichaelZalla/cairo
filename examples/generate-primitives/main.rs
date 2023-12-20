extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    image::TextureMap,
    material::Material,
    mesh,
    scene::Scene,
};

mod generate_primitives_scene;

use self::generate_primitives_scene::GeneratePrimitivesScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static WINDOW_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/generate-primitives", WINDOW_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Generate primitive meshes
    let mut plane_mesh = mesh::primitive::plane::generate(30.0, 30.0, 8, 8);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);
    let mut cone_mesh = mesh::primitive::cone::generate(2.0, 2.0, 40);
    let mut cylinder_mesh = mesh::primitive::cylinder::generate(2.0, 2.0, 40);

    // Create a new textured material
    let mut checkerboard_mat = Material::new("checkerboard".to_string());

    let mut checkerboard_texture =
        TextureMap::new(&"./examples/generate-primitives/assets/checkerboard.png");

    checkerboard_texture.load(rendering_context)?;

    checkerboard_mat.diffuse_map = Some(checkerboard_texture);

    plane_mesh.material = Some(checkerboard_mat.clone());
    cube_mesh.material = Some(checkerboard_mat.clone());
    cone_mesh.material = Some(checkerboard_mat.clone());
    cylinder_mesh.material = Some(checkerboard_mat.clone());

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.x -= 3.0;
    cube_entity.position.y -= 1.5;

    let mut cone_entity = Entity::new(&cone_mesh);
    // cone_entity.position.x += 1.5;
    cone_entity.position.y -= 1.5;

    let mut cylinder_entity = Entity::new(&cylinder_mesh);
    cylinder_entity.position.x += 3.0;
    cylinder_entity.position.y -= 1.5;

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![
        &mut plane_entity,
        &mut cube_entity,
        &mut cone_entity,
        &mut cylinder_entity,
    ];

    let entities_rwl = RwLock::new(entities);

    // Instantiate our textured cube scene
    let scene = RefCell::new(GeneratePrimitivesScene::new(
        rendering_context,
        &entities_rwl,
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
