extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    font::{cache::FontCache, FontInfo},
    material::{cache::MaterialCache, Material},
    mesh,
    scene::Scene,
    shader::ShaderContext,
    texture::TextureMap,
    time::TimingInfo,
};

mod generate_primitives_scene;

use self::generate_primitives_scene::GeneratePrimitivesScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/generate-primitives", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Fonts

    let font_info = FontInfo {
        filepath: "C:/Windows/Fonts/vgasys.fon".to_string(),
        point_size: 16,
    };

    let font_cache_rwl = RwLock::new(FontCache::new(app.context.ttf_context));

    font_cache_rwl.write().unwrap().load(&font_info)?;

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(32.0, 32.0, 1, 1);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);
    let mut cone_mesh = mesh::primitive::cone::generate(2.0, 2.0, 40);
    let mut cylinder_mesh = mesh::primitive::cylinder::generate(2.0, 2.0, 40);

    // Create a new textured material
    let mut checkerboard_mat = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(&"./assets/textures/checkerboard.jpg");

    // Checkerboard material

    checkerboard_diffuse_map.is_tileable = true;

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    // Pump up diffuse value of the darkest pixels
    checkerboard_diffuse_map.map(|r, g, b| {
        if r < 4 && g < 4 && b < 4 {
            return (18, 18, 18);
        }
        (r, g, b)
    })?;

    checkerboard_mat.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_mat.specular_exponent = 8;

    checkerboard_mat.specular_map = Some(checkerboard_specular_map);

    // Point light decal material

    let mut point_light_decal_mat = Material::new("point_light_decal".to_string());

    point_light_decal_mat.alpha_map =
        Some(TextureMap::new(&"./assets/decals/point_light_small.png"));

    point_light_decal_mat.emissive_map = point_light_decal_mat.alpha_map.clone();

    point_light_decal_mat.load_all_maps(rendering_context)?;

    // Spot light decal material

    let mut spot_light_decal_mat = Material::new("spot_light_decal".to_string());

    spot_light_decal_mat.alpha_map = Some(TextureMap::new(&"./assets/decals/spot_light_small.png"));

    spot_light_decal_mat.emissive_map = spot_light_decal_mat.alpha_map.clone();

    spot_light_decal_mat.load_all_maps(rendering_context)?;

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_mat.name.clone());
    cube_mesh.material_name = Some(checkerboard_mat.name.clone());
    cone_mesh.material_name = Some(checkerboard_mat.name.clone());
    cylinder_mesh.material_name = Some(checkerboard_mat.name.clone());

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    plane_entity.position.x -= 5.0;
    plane_entity.position.z -= 5.0;

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.x -= 4.0;
    cube_entity.position.y += 1.0;

    let mut cone_entity = Entity::new(&cone_mesh);
    cone_entity.position.x -= 0.0;
    cone_entity.position.y += 1.0;

    let mut cylinder_entity = Entity::new(&cylinder_mesh);
    cylinder_entity.position.x += 4.0;
    cylinder_entity.position.y += 1.0;

    material_cache.insert(checkerboard_mat);
    material_cache.insert(point_light_decal_mat);
    material_cache.insert(spot_light_decal_mat);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![
        &mut plane_entity,
        &mut cube_entity,
        &mut cone_entity,
        &mut cylinder_entity,
    ];

    let entities_rwl = RwLock::new(entities);

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    // Instantiate our textured cube scene
    let scene = RefCell::new(GeneratePrimitivesScene::new(
        app.canvas_width,
        app.canvas_height,
        &font_cache_rwl,
        &font_info,
        &entities_rwl,
        &mut material_cache,
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
