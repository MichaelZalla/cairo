extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    font::{cache::FontCache, FontInfo},
    mesh,
    scene::Scene,
    shader::ShaderContext,
};

mod sponza_scene;

use self::sponza_scene::SponzaScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/sponza".to_string(),
        window_width: 860,
        window_height: 520,
        canvas_width: 860,
        canvas_height: 520,
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Fonts

    let font_info = Box::leak(Box::new(FontInfo {
        filepath: "C:/Windows/Fonts/vgasys.fon".to_string(),
        point_size: 16,
    }));

    let font_cache_rwl = Box::leak(Box::new(RwLock::new(FontCache::new(
        app.context.ttf_context,
    ))));

    font_cache_rwl.write().unwrap().load(&font_info)?;

    // Default framebuffer

    let framebuffer_rwl = RwLock::new(Buffer2D::new(
        window_info.canvas_width,
        window_info.canvas_height,
        None,
    ));

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
        &framebuffer_rwl,
        font_cache_rwl,
        font_info,
        rendering_context,
        &entities_rwl,
        &mut materials,
        &shader_context_rwl,
    ));

    // Set up our app
    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> () {
        // Delegate the update to our spinning cube scene
        scene
            .borrow_mut()
            .update(app, keyboard_state, mouse_state, game_controller_state);
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our spinning cube scene

        scene.borrow_mut().render();

        let framebuffer = framebuffer_rwl.read().unwrap();

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
