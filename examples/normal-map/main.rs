extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::{App, AppWindowInfo},
    buffer::Buffer2D,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::{cache::MaterialCache, Material},
    mesh,
    scene::Scene,
    shader::ShaderContext,
    texture::TextureMap,
};

mod normal_map_scene;

use self::normal_map_scene::NormalMapScene;

fn main() -> Result<(), String> {
    let resolutions = vec![
        // (320, 180),
        // (640, 320),
        // (800, 450),
        // (960, 540),
        // (1024, 576),
        (1200, 675),
        // (1280, 720),
        // (1366, 768),
        // (1920, 1080),
        // (2560, 1440),
    ];

    let resolution = resolutions[0];

    let mut window_info = AppWindowInfo {
        title: "examples/normal-map".to_string(),
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

    let framebuffer_rwl = RwLock::new(Buffer2D::new(
        window_info.canvas_width,
        window_info.canvas_height,
        None,
    ));

    // Brick wall mesh

    let (brick_wall_meshes, _) = mesh::obj::load_obj("./data/obj/sphere.obj");
    let mut brick_wall_mesh = brick_wall_meshes[0].to_owned();

    // Brick pattern

    let mut brick_material = Material::new("brick".to_string());

    brick_material.specular_exponent = 32;

    let brick_diffuse_map =
        TextureMap::new(&"./examples/normal-map/assets/Brick_OldDestroyed_1k_d.tga");

    let brick_specular_map =
        TextureMap::new(&"./examples/normal-map/assets/Brick_OldDestroyed_1k_s.tga");

    let brick_normal_map =
        TextureMap::new(&"./examples/normal-map/assets/Brick_OldDestroyed_1k_nY+.tga");

    brick_material.diffuse_map = Some(brick_diffuse_map);
    brick_material.specular_map = Some(brick_specular_map);
    brick_material.normal_map = Some(brick_normal_map);

    brick_material.load_all_maps(rendering_context)?;

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    // Assign textures to mesh materials

    brick_wall_mesh.material_name = Some(brick_material.name.to_string());

    material_cache.insert(brick_material);

    // Assign the meshes to entities

    let mut brick_wall_entity: Entity<'_> = Entity::new(&brick_wall_mesh);

    // Wrap the entity collection in a memory-safe container

    let entities: Vec<&mut Entity> = vec![&mut brick_wall_entity];

    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our scene

    let scene = RefCell::new(NormalMapScene::new(
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
     -> () {
        // Delegate the update to our textured cube scene

        scene
            .borrow_mut()
            .update(&app, &keyboard_state, &mouse_state, &game_controller_state);
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our textured cube scene

        scene.borrow_mut().render();

        let framebuffer = framebuffer_rwl.read().unwrap();

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}