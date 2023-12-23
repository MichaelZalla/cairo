extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    image::TextureMap,
    material::Material,
    mesh,
    scene::Scene,
    vec::vec3::Vec3,
};

mod specular_map_scene;

use self::specular_map_scene::SpecularMapScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;

static CANVAS_WIDTH: u32 = 960;

fn main() -> Result<(), String> {
    let app = App::new("examples/specular-map", CANVAS_WIDTH, ASPECT_RATIO);

    let rendering_context = &app.context.rendering_context;

    // Generate primitive meshes

    let mut plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

    // Initialize materials

    // Checkerboard pattern

    let mut checkerboard_material = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map =
        TextureMap::new(&"./examples/specular-map/assets/checkerboard.jpg");

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_material.specular_map = Some(checkerboard_specular_map);

    // Metal-wrapped wooden container

    let mut container_material = Material::new("container".to_string());

    let mut container_diffuse_map =
        TextureMap::new(&"./examples/specular-map/assets/container2.png");

    let mut container_specular_map =
        TextureMap::new(&"./examples/specular-map/assets/container2_specular.png");

    container_diffuse_map.load(rendering_context)?;

    container_material.diffuse_map = Some(container_diffuse_map);

    container_specular_map.load(rendering_context)?;

    container_material.specular_map = Some(container_specular_map);

    // Assign textures to mesh materials

    plane_mesh.material = Some(checkerboard_material);

    cube_mesh.material = Some(container_material);

    // Assign the meshes to entities
    let mut plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.y -= 1.5;

    // Orbiting point light

    let mut point_light_material = Material::new("white".to_string());
    point_light_material.diffuse_color = color::WHITE.to_vec3() / 255.0;

    let mut point_light_mesh = mesh::primitive::cube::generate(0.2, 0.2, 0.2);

    point_light_mesh.object_name = "point_light".to_string();
    point_light_mesh.material = Some(point_light_material);

    let mut point_light_entity = Entity::new(&point_light_mesh);

    point_light_entity.position = Vec3 {
        x: 0.0,
        y: -5.0,
        z: 0.0,
    };

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> =
        vec![&mut plane_entity, &mut cube_entity, &mut point_light_entity];

    let entities_rwl = RwLock::new(entities);

    // Instantiate our textured cube scene
    let scene = RefCell::new(SpecularMapScene::new(
        app.canvas_width,
        app.canvas_height,
        &entities_rwl,
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
