extern crate sdl2;

use std::{cell::RefCell, path::Path, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material, mesh,
    scene::Scene,
};

mod texture_mapped_cube_scene;

use self::texture_mapped_cube_scene::TextureMappedCubeScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 800;

fn main() -> Result<(), String> {
    let app = App::new("examples/texture-mapped-cube", WINDOW_WIDTH, ASPECT_RATIO);

    // Load a cube mesh
    let cube_meshes = mesh::obj::load_obj("./data/obj/cube-textured.obj".to_string());
    let cube_mesh = &cube_meshes[0];

    if cube_mesh.material_source.filepath.len() > 0 {
        let cube_object_source_parent = Path::new(&cube_mesh.object_source).parent().unwrap();

        let cube_material_source = Path::new(&cube_mesh.material_source.filepath);

        let cube_material_source_path_relative = cube_object_source_parent
            .join(cube_material_source)
            .into_os_string()
            .into_string()
            .unwrap();

        let cube_materials = material::mtl::load_mtl(cube_material_source_path_relative);
    }

    // Assign the mesh to a new entity
    let mut cube_entity = Entity::new(&cube_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);

    let rendering_context = &app.context.rendering_context;

    // Instantiate our textured cube scene
    let scene = RefCell::new(TextureMappedCubeScene::new(
        rendering_context,
        &entities_rwl,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      delta_t_seconds: f32|
     -> () {
        // Delegate the update to our textured cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            delta_t_seconds,
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
