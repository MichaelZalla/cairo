extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    mesh,
    scene::Scene,
};

mod spinning_cube_scene;

use self::spinning_cube_scene::SpinningCubeScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    // Generate a cube mesh
    let cube_mesh = mesh::obj::get_mesh_from_obj("./data/obj/cube.obj".to_string());
    // let cube_mesh = mesh::primitive::make_box(1.0, 1.0, 1.0);

    // Assign the mesh to a new entity
    let mut cube_entity = Entity::new(&cube_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);

    // Instantiate our spinning cube scene
    let scene = RefCell::new(SpinningCubeScene::new(
        Graphics {
            buffer: PixelBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        },
        &entities_rwl,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      delta_t_seconds: f32|
     -> () {
        // Delegate the update to our spinning cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            delta_t_seconds,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our spinning cube scene

        scene.borrow_mut().render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    let app = App::new("examples/spinning-cube", WINDOW_WIDTH, ASPECT_RATIO);

    app.run(&mut update, &mut render)?;

    Ok(())
}
