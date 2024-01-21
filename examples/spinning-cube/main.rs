extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh,
    scene::Scene,
    shader::ShaderContext,
    time::TimingInfo,
};

mod spinning_cube_scene;

use self::spinning_cube_scene::SpinningCubeScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/spinning-cube".to_string(),
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Generate a cube mesh
    let cube_mesh = mesh::primitive::cube::generate(1.0, 1.0, 1.0);

    // Assign the mesh to a new entity
    let mut cube_entity = Entity::new(&cube_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our spinning cube scene
    let scene = RefCell::new(SpinningCubeScene::new(
        window_info.canvas_width,
        window_info.canvas_height,
        &entities_rwl,
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
            timing_info,
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
