extern crate sdl2;

use std::{cell::RefCell, cmp::min, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    fs::get_absolute_filepath,
    graphics::{Graphics, PixelBuffer},
    matrix::Mat4,
    mesh::obj::get_mesh_from_obj,
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight},
        Scene,
    },
    scenes::default_scene::DefaultScene,
    vec::{vec3::Vec3, vec4::Vec4},
};
use sdl2::keyboard::Keycode;

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    // Import mesh data
    let cube_mesh = get_mesh_from_obj(get_absolute_filepath("./data/obj/cube.obj"));
    let teapot_mesh = get_mesh_from_obj(get_absolute_filepath("./data/obj/teapot.obj"));

    // Assign meshes to new entities
    let mut cube_entity = Entity::new(&cube_mesh);
    let mut teapot_entity = Entity::new(&teapot_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);
    let entities2 = vec![&mut teapot_entity];
    let entities2_rwl = RwLock::new(entities2);

    // Set up a camera for rendering our scenes
    let camera: Camera = Camera::new(
        Vec4::new(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            1.0,
        ),
        Mat4::identity(),
        150.0,
        0.0,
        6.0,
    );

    // Define (shared) lights for our scenes
    let ambient_light = AmbientLight {
        intensities: Vec3 {
            x: 0.1,
            y: 0.1,
            z: 0.1,
        },
    };

    let directional_light = DirectionalLight {
        intensities: Vec3 {
            x: 0.3,
            y: 0.3,
            z: 0.3,
        },
        direction: Vec4 {
            x: 0.25,
            y: -1.0,
            z: -0.25,
            w: 1.0,
        },
    };

    let point_light = PointLight {
        intensities: Vec3 {
            x: 0.4,
            y: 0.4,
            z: 0.4,
        },
        position: Vec3::new(),
        constant_attenuation: 0.382,
        linear_attenuation: 1.0,
        quadratic_attenuation: 2.619,
    };

    let graphics = Graphics {
        buffer: PixelBuffer {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            width_over_height: ASPECT_RATIO,
            height_over_width: 1.0 / ASPECT_RATIO,
            pixels: vec![0 as u32; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize],
        },
    };

    let scenes = RefCell::new(vec![
        DefaultScene::new(
            graphics.clone(),
            camera,
            ambient_light,
            directional_light,
            point_light,
            &entities_rwl,
        ),
        DefaultScene::new(
            graphics.clone(),
            camera,
            ambient_light,
            directional_light,
            point_light,
            &entities2_rwl,
        ),
    ]);

    let current_scene_index = RefCell::new(min(0, scenes.borrow().len() - 1));

    // Set up our app

    let update = |keyboard_state: &KeyboardState,
                  mouse_state: &MouseState,
                  game_controller_state: &GameControllerState,
                  delta_t_seconds: f32|
     -> () {
        // Update scene

        let scenes_len = scenes.borrow_mut().len();

        let mut new_index = *current_scene_index.borrow();

        for keycode in keyboard_state.keys_pressed.to_owned() {
            match keycode {
                Keycode::Num4 { .. } => {
                    new_index = min(scenes_len - 1, 0);
                }
                Keycode::Num5 { .. } => {
                    new_index = min(scenes_len - 1, 1);
                }
                _ => {}
            }
        }

        *current_scene_index.borrow_mut() = new_index;

        scenes.borrow_mut()[*current_scene_index.borrow()].update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            delta_t_seconds,
        );
    };

    let render = || -> Result<Vec<u32>, String> {
        // Render current scene

        scenes.borrow_mut()[*current_scene_index.borrow()].render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scenes.borrow_mut()[*current_scene_index.borrow()]
            .get_pixel_data()
            .clone());
    };

    let app = App::new(
        "examples/multiple-scenes",
        WINDOW_WIDTH,
        ASPECT_RATIO,
        update,
        render,
    );

    app.run()?;

    Ok(())
}
