extern crate sdl2;

use std::{cell::RefCell, cmp::min, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh::obj::load_obj,
    scene::Scene,
    shader::ShaderContext,
};

mod multiple_scenes_scene;

use multiple_scenes_scene::MultipleScenesScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/multiple-scenes".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Default framebuffer

    let framebuffer_rwl = RwLock::new(Buffer2D::new(
        window_info.canvas_width,
        window_info.canvas_height,
        None,
    ));

    // Load meshes
    let (cube_meshes, _cube_materials) = load_obj(&"./data/obj/cube.obj");
    let cube_mesh = &cube_meshes[0];

    let (teapot_meshes, _teapot_materials) = load_obj(&"./data/obj/teapot.obj");
    let teapot_mesh = &teapot_meshes[0];

    // Assign meshes to new entities
    let mut cube_entity = Entity::new(&cube_mesh);
    let mut teapot_entity = Entity::new(&teapot_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);
    let entities2 = vec![&mut teapot_entity];
    let entities2_rwl = RwLock::new(entities2);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    let scenes = RefCell::new(vec![
        MultipleScenesScene::new(&framebuffer_rwl, &entities_rwl, &shader_context_rwl),
        MultipleScenesScene::new(&framebuffer_rwl, &entities2_rwl, &shader_context_rwl),
    ]);

    let current_scene_index = RefCell::new(min(0, scenes.borrow().len() - 1));

    // Set up our app

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
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
            app,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Render current scene

        scenes.borrow_mut()[*current_scene_index.borrow()].render();

        let framebuffer = framebuffer_rwl.read().unwrap();

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
