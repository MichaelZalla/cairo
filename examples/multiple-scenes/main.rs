extern crate sdl2;

use std::{cell::RefCell, cmp::min, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    mesh::obj::load_obj,
    scene::Scene,
};

mod multiple_scenes_scene;

use multiple_scenes_scene::MultipleScenesScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    // Load meshes
    let (cube_meshes, cube_materials) = load_obj(&"./data/obj/cube.obj");
    let cube_mesh = &cube_meshes[0];

    let (teapot_meshes, teapot_materials) = load_obj(&"./data/obj/teapot.obj");
    let teapot_mesh = &teapot_meshes[0];

    // Assign meshes to new entities
    let mut cube_entity = Entity::new(&cube_mesh);
    let mut teapot_entity = Entity::new(&teapot_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);
    let entities2 = vec![&mut teapot_entity];
    let entities2_rwl = RwLock::new(entities2);

    let graphics = Graphics {
        buffer: PixelBuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT),
    };

    let scenes = RefCell::new(vec![
        MultipleScenesScene::new(graphics.clone(), &entities_rwl),
        MultipleScenesScene::new(graphics.clone(), &entities2_rwl),
    ]);

    let current_scene_index = RefCell::new(min(0, scenes.borrow().len() - 1));

    // Set up our app

    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      seconds_since_last_update: f32|
     -> () {
        // Update scene

        let scenes_len = scenes.borrow_mut().len();

        let mut new_index = *current_scene_index.borrow();

        for keycode in keyboard_state.keys_pressed.to_owned() {
            match keycode {
                Keycode::Num5 { .. } => {
                    new_index = min(scenes_len - 1, 0);
                }
                Keycode::Num6 { .. } => {
                    new_index = min(scenes_len - 1, 1);
                }
                _ => {}
            }
        }

        *current_scene_index.borrow_mut() = new_index;

        scenes.borrow_mut()[*current_scene_index.borrow()].update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            seconds_since_last_update,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Render current scene

        scenes.borrow_mut()[*current_scene_index.borrow()].render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scenes.borrow_mut()[*current_scene_index.borrow()]
            .get_pixel_data()
            .clone());
    };

    let app = App::new("examples/multiple-scenes", WINDOW_WIDTH, ASPECT_RATIO);

    app.run(&mut update, &mut render)?;

    Ok(())
}
