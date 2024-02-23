extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh,
    scene::Scene,
    shader::context::ShaderContext,
};

mod diffuse_map_scene;

use self::diffuse_map_scene::DiffuseMapScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/diffuse-map".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(window_info.canvas_width, window_info.canvas_height);

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rwl = RwLock::new(framebuffer);

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

    // Instantiate our scene
    let scene: RefCell<DiffuseMapScene<'_>> = RefCell::new(DiffuseMapScene::new(
        &framebuffer_rwl,
        &entities_rwl,
        &cache,
        &shader_context_rwl,
    ));

    // Set up our app
    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        // Delegate the update to our scene.

        scene
            .borrow_mut()
            .update(app, keyboard_state, mouse_state, game_controller_state);

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our scene.

        scene.borrow_mut().render();

        let framebuffer = framebuffer_rwl.read().unwrap();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let color_buffer = color_buffer_lock.read().unwrap();

                return Ok(color_buffer.get_all().clone());
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
