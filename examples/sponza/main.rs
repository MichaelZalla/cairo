extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    mesh,
    scene::Scene,
};

mod sponza_scene;

use self::sponza_scene::SponzaScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;
static CANVAS_HEIGHT: u32 = (CANVAS_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    let app = App::new("examples/sponza", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    let (atrium_meshes, mut atrium_materials) =
        mesh::obj::load_obj("./examples/sponza/assets/sponza.obj");

    // Load material maps

    match &mut atrium_materials {
        Some(cache) => {
            for material in cache.values_mut() {
                material.load_all_maps(rendering_context)?;
            }
        }
        None => (),
    }

    // Create one entity per mesh that we parsed

    let mut entities: Vec<Entity> = vec![];

    for i in 0..atrium_meshes.len() {
        entities.push(Entity::new(&atrium_meshes[i]));
    }

    // Wrap the entity collection in a memory-safe container
    let entities_rwl = RwLock::new(entities);

    let materials = atrium_materials.unwrap();

    // Instantiate our spinning cube scene
    let scene = RefCell::new(SponzaScene::new(
        Graphics {
            buffer: PixelBuffer::new(CANVAS_WIDTH, CANVAS_HEIGHT),
        },
        rendering_context,
        &entities_rwl,
        &materials,
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

        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}