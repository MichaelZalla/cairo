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
    shader::ShaderContext,
    texture::map::{TextureMap, TextureMapStorageFormat},
};

mod specular_map_scene;

use self::specular_map_scene::SpecularMapScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/specular-map".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(window_info.canvas_width, window_info.canvas_height);

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rwl = RwLock::new(framebuffer);

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

    // Initialize materials

    // Checkerboard pattern

    let mut checkerboard_material = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(
        &"./assets/textures/checkerboard.jpg",
        TextureMapStorageFormat::Index8,
    );

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_material.specular_map = Some(checkerboard_specular_map);

    // Metal-wrapped wooden container

    let mut container_material = Material::new("container".to_string());

    let container_diffuse_map = TextureMap::new(
        &"./examples/specular-map/assets/container2.png",
        TextureMapStorageFormat::RGB24,
    );

    let container_specular_map = TextureMap::new(
        &"./examples/specular-map/assets/container2_specular.png",
        TextureMapStorageFormat::Index8,
    );

    container_material.diffuse_map = Some(container_diffuse_map);

    container_material.specular_map = Some(container_specular_map);

    container_material.load_all_maps(rendering_context).unwrap();

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_material.name.clone());

    cube_mesh.material_name = Some(container_material.name.clone());

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    material_cache.insert(checkerboard_material);

    material_cache.insert(container_material);

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.y += 1.5;

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut plane_entity, &mut cube_entity];

    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our scene
    let scene = RefCell::new(SpecularMapScene::new(
        &framebuffer_rwl,
        &entities_rwl,
        &material_cache,
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
            .update(app, keyboard_state, mouse_state, game_controller_state);
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our textured cube scene

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
