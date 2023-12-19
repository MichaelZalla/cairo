extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh,
    scene::Scene,
};

mod generate_primitives_scene;

use self::generate_primitives_scene::GeneratePrimitivesScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static WINDOW_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/generate-primitives", WINDOW_WIDTH, ASPECT_RATIO);

    // Generate a cube mesh
    let plane_mesh = mesh::primitive::plane::generate(30.0, 30.0, 10, 10);
    let cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);
    let cone_mesh = mesh::primitive::cone::generate(2.0, 2.0, 60);

    // Assign the mesh to new entities
    let mut plane_entity = Entity::new(&plane_mesh, None);

    let mut cube_entity = Entity::new(&cube_mesh, None);
    cube_entity.position.x -= 1.5;
    cube_entity.position.y -= 1.5;

    let mut cone_entity = Entity::new(&cone_mesh, None);
    cone_entity.position.x += 1.5;
    cone_entity.position.y -= 1.5;

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut plane_entity, &mut cube_entity, &mut cone_entity];

    let entities_rwl = RwLock::new(entities);

    let rendering_context = &app.context.rendering_context;

    // Instantiate our textured cube scene
    let scene = RefCell::new(GeneratePrimitivesScene::new(
        rendering_context,
        &entities_rwl,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      delta_t_seconds: f32|
     -> () {
        // Delegate the update to our textured cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            delta_t_seconds,
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
