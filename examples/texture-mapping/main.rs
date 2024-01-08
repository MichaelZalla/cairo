extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    mesh,
    scene::Scene,
};

mod texture_mapped_cube_scene;

use self::texture_mapped_cube_scene::TextureMappedCubeScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/texture-mapped-cube", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Load a cube mesh and its materials

    let (mut cube_meshes, mut cube_material_cache) =
        mesh::obj::load_obj(&"./data/obj/cube-textured.obj");

    let cube_mesh = &mut cube_meshes[0];

    match &mut cube_material_cache {
        Some(cache) => {
            for material in cache.values_mut() {
                // Ambient color map
                match &mut material.ambient_map {
                    Some(map) => {
                        map.load(rendering_context)?;
                    }
                    None => (),
                }

                // Diffuse color map
                match &mut material.diffuse_map {
                    Some(map) => {
                        map.load(rendering_context)?;
                    }
                    None => (),
                }

                // Normal map
                match &mut material.normal_map {
                    Some(map) => {
                        map.load(rendering_context)?;
                    }
                    None => (),
                }
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

    // Instantiate our textured cube scene
    let scene = RefCell::new(TextureMappedCubeScene::new(
        rendering_context,
        &entities_rwl,
        &cache,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      seconds_since_last_update: f32|
     -> () {
        // Delegate the update to our textured cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            seconds_since_last_update,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our textured cube scene

        scene.borrow_mut().render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
