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

mod texture_mapped_cube_scene;

use self::texture_mapped_cube_scene::TextureMappedCubeScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/texture-mapping".to_string(),
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Load a cube mesh and its materials

    let (mut cube_meshes, mut cube_material_cache) =
        mesh::obj::load_obj(&"./data/obj/cube-textured.obj");

    let cube_mesh = &mut cube_meshes[0];

    match &mut cube_material_cache {
        Some(cache) => {
            for material in cache.values_mut() {
                material.load_all_maps(rendering_context).unwrap();
            }
        }
        None => (),
    }

    // Assign the mesh to a new entity
    let mut cube_entity = Entity::new(&cube_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);

    let cache = cube_material_cache.unwrap();

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our textured cube scene
    let scene: RefCell<TextureMappedCubeScene<'_>> = RefCell::new(TextureMappedCubeScene::new(
        window_info.canvas_width,
        window_info.canvas_height,
        &entities_rwl,
        &cache,
        &shader_context_rwl,
    ));

    // Set up our app
    let mut update = |timing_info: &TimingInfo,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
        // Delegate the update to our textured cube scene

        scene.borrow_mut().update(
            &timing_info,
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our textured cube scene

        scene.borrow_mut().render();

        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
