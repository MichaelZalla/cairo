extern crate sdl2;

use std::{cell::RefCell, sync::RwLock};

use cairo::{
    app::App,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    matrix::Mat4,
    mesh,
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight},
        Scene,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

mod spinning_cube_scene;

use self::spinning_cube_scene::SpinningCubeScene;

static ASPECT_RATIO: f32 = 16.0 / 9.0;
static WINDOW_WIDTH: u32 = 1080;
static WINDOW_HEIGHT: u32 = (WINDOW_WIDTH as f32 / ASPECT_RATIO) as u32;

fn main() -> Result<(), String> {
    // Generate a cube mesh
    let cube_mesh = mesh::obj::get_mesh_from_obj("./data/obj/cube.obj".to_string());
    // let cube_mesh = mesh::primitive::make_box(1.0, 1.0, 1.0);

    // Assign the mesh to a new entity
    let mut cube_entity = Entity::new(&cube_mesh);

    // Wrap the entity collection in a memory-safe container
    let entities: Vec<&mut Entity> = vec![&mut cube_entity];
    let entities_rwl = RwLock::new(entities);

    // Set up a camera for rendering our cube scene
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

    // Define lights for our scene
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
        position: Default::default(),
        constant_attenuation: 0.382,
        linear_attenuation: 1.0,
        quadratic_attenuation: 2.619,
    };

    // Instantiate our spinning cube scene
    let scene = RefCell::new(SpinningCubeScene::new(
        Graphics {
            buffer: PixelBuffer {
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
                width_over_height: ASPECT_RATIO,
                height_over_width: 1.0 / ASPECT_RATIO,
                pixels: vec![0 as u32; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize],
            },
        },
        camera,
        ambient_light,
        directional_light,
        point_light,
        &entities_rwl,
    ));

    // Set up our app
    let mut update = |keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState,
                      delta_t_seconds: f32|
     -> () {
        // Delegate the update to our spinning cube scene

        scene.borrow_mut().update(
            &keyboard_state,
            &mouse_state,
            &game_controller_state,
            delta_t_seconds,
        );
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our spinning cube scene

        scene.borrow_mut().render();

        // @TODO(mzalla) Return reference to a captured variable???
        return Ok(scene.borrow_mut().get_pixel_data().clone());
    };

    let app = App::new("examples/spinning-cube", WINDOW_WIDTH, ASPECT_RATIO);

    app.run(&mut update, &mut render)?;

    Ok(())
}
