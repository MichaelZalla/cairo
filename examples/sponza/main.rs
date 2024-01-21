extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh,
    scene::Scene,
    shader::ShaderContext,
    time::TimingInfo,
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

    let mut materials = atrium_materials.unwrap();

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our spinning cube scene
    let scene = RefCell::new(SponzaScene::new(
        CANVAS_WIDTH,
        CANVAS_HEIGHT,
        rendering_context,
        &entities_rwl,
        &mut materials,
        &shader_context_rwl,
    ));

    // Set up our app
    let mut update = |timing_info: &TimingInfo,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
        // Delegate the update to our spinning cube scene
        scene.borrow_mut().update(
            &timing_info,
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
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
