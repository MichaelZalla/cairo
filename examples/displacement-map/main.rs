extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::{cache::MaterialCache, Material},
    mesh,
    scene::Scene,
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::vec3::Vec3,
};

mod displacement_map_scene;

use self::displacement_map_scene::DisplacementMapScene;

fn main() -> Result<(), String> {
    let resolutions = vec![
        // (320, 180),
        // (640, 320),
        // (800, 450),
        (960, 540),
        // (1024, 576),
        // (1200, 675),
        // (1280, 720),
        // (1366, 768),
        // (1920, 1080),
        // (2560, 1440),
    ];

    let resolution = resolutions[0];

    let mut window_info = AppWindowInfo {
        title: "examples/diplacement-map".to_string(),
        full_screen: false,
        vertical_sync: true,
        relative_mouse_mode: true,
        window_width: resolution.0,
        window_height: resolution.1,
        canvas_width: resolution.0,
        canvas_height: resolution.1,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(window_info.canvas_width, window_info.canvas_height);

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rwl = RwLock::new(framebuffer);

    // Meshes

    let mut brick_wall_mesh = mesh::primitive::cube::generate(4.0, 4.0, 4.0);

    let mut box_mesh = brick_wall_mesh.clone();

    // Bricks material

    let mut brick_material = Material::new("bricks".to_string());

    brick_material.diffuse_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/bricks2.jpg",
        TextureMapStorageFormat::RGB24,
    ));

    brick_material.normal_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/bricks2_normal.jpg",
        TextureMapStorageFormat::RGB24,
    ));

    brick_material.displacement_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/bricks2_disp.jpg",
        TextureMapStorageFormat::Index8,
    ));

    brick_material.displacement_scale = 0.05;

    brick_material.load_all_maps(rendering_context)?;

    // Box material

    let mut box_material = Material::new("box".to_string());

    box_material.diffuse_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/wood.png",
        TextureMapStorageFormat::RGB24,
    ));
    box_material.normal_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/toy_box_normal.png",
        TextureMapStorageFormat::RGB24,
    ));
    box_material.displacement_map = Some(TextureMap::new(
        &"./examples/displacement-map/assets/toy_box_disp.png",
        TextureMapStorageFormat::Index8,
    ));

    box_material.load_all_maps(rendering_context)?;

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    // Assign textures to mesh materials

    brick_wall_mesh.material_name = Some(brick_material.name.to_string());
    box_mesh.material_name = Some(box_material.name.to_string());

    material_cache.insert(brick_material);
    material_cache.insert(box_material);

    // Assign the meshes to entities

    let mut brick_wall_entity: Entity<'_> = Entity::new(&brick_wall_mesh);

    brick_wall_entity.position = Vec3 {
        x: -4.0,
        y: 0.0,
        z: 2.0,
    };

    // brick_wall_entity.rotation.x = -1.0 * PI / 2.0;

    let mut box_entity: Entity<'_> = Entity::new(&box_mesh);

    box_entity.position = Vec3 {
        x: 4.0,
        y: 0.0,
        z: 2.0,
    };

    // Wrap the entity collection in a memory-safe container

    let entities: Vec<&mut Entity> = vec![&mut brick_wall_entity, &mut box_entity];

    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our scene

    let scene = RefCell::new(DisplacementMapScene::new(
        &framebuffer_rwl,
        &entities_rwl,
        &mut material_cache,
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
            .update(&app, &keyboard_state, &mouse_state, &game_controller_state);

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
