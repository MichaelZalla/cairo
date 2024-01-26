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
    time::TimingInfo,
};

mod emissive_map_scene;

use self::emissive_map_scene::EmissiveMapScene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/emissive-map".to_string(),
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

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

    // Initialize materials

    // Checkerboard material

    let mut checkerboard_material = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(&"./assets/textures/checkerboard.jpg");

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_material.specular_map = Some(checkerboard_specular_map);

    // Laval material

    let mut lava_material = Material::new("container".to_string());

    let lava_diffuse_map = TextureMap::new(&"./examples/emissive-map/assets/lava.png");

    let lava_emissive_map = TextureMap::new(&"./examples/emissive-map/assets/lava_emissive.png");

    lava_material.diffuse_map = Some(lava_diffuse_map);

    lava_material.emissive_map = Some(lava_emissive_map);

    lava_material.load_all_maps(rendering_context).unwrap();

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_material.name.clone());

    cube_mesh.material_name = Some(lava_material.name.clone());

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    material_cache.insert(checkerboard_material);

    material_cache.insert(lava_material);

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.y = 3.0;

    // Orbiting point light

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut plane_entity, &mut cube_entity];

    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our textured cube scene
    let scene = RefCell::new(EmissiveMapScene::new(
        &framebuffer_rwl,
        &entities_rwl,
        &material_cache,
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
            timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );
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
