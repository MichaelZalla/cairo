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

mod skybox_scene;

use self::skybox_scene::SkyboxScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;
static CANVAS_HEIGHT: u32 = (CANVAS_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    let app = App::new("examples/skybox", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Generate a cube mesh
    let cube_mesh = mesh::primitive::cube::generate(1.0, 1.0, 1.0);

    // Assign the mesh to a new entity
    let mut cube_entity = Entity::new(&cube_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);

    // Instantiate our spinning cube scene
    let scene = RefCell::new(SkyboxScene::new(
        Graphics {
            buffer: PixelBuffer::new(CANVAS_WIDTH, CANVAS_HEIGHT),
        },
        rendering_context,
        &entities_rwl,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      seconds_since_last_update: f32|
     -> () {
        // Delegate the update to our spinning cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            seconds_since_last_update,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our spinning cube scene

        scene.borrow_mut().render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}