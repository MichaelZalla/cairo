extern crate sdl2;

use std::cell::RefCell;

use cairo::{
    app::{resolution::Resolution, App, AppWindowInfo},
    buffer::Buffer2D,
    color::{self},
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    graphics::Graphics,
    vec::vec3::Vec3,
};

use random::{DirectionSampler, RandomSampler};

mod random;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/particle-systems".to_string(),
        ..Default::default()
    };

    let render_scene_to_framebuffer = |_frame_index: Option<u32>,
                                       _new_resolution: Option<Resolution>|
     -> Result<Vec<u32>, String> { Ok(vec![]) };

    let (app, _event_watch) = App::new(&mut window_info, &render_scene_to_framebuffer);

    // Set up our app

    let framebuffer = Buffer2D::new(
        window_info.window_resolution.width,
        window_info.window_resolution.height,
        None,
    );

    let framebuffer_center = Vec3 {
        x: framebuffer.width as f32 / 2.0,
        y: framebuffer.height as f32 / 2.0,
        z: 0.0,
    };

    let framebuffer_rc = RefCell::new(framebuffer);

    let sampler_rc: RefCell<RandomSampler> = Default::default();

    let mut update = |_app: &mut App,
                      _keyboard_state: &mut KeyboardState,
                      _mouse_state: &mut MouseState,
                      _game_controller_state: &mut GameControllerState|
     -> Result<(), String> { Ok(()) };

    let render = |_frame_index, _new_resolution| -> Result<Vec<u32>, String> {
        let mut sampler = sampler_rc.borrow_mut();
        let mut framebuffer = framebuffer_rc.borrow_mut();

        // Clears pixel buffer

        // framebuffer.clear(Some(color::BLACK.to_u32()));

        {
            let uniform_sample_normal = sampler.sample_direction_uniform();

            let start = framebuffer_center;
            let end = framebuffer_center + uniform_sample_normal * 200.0;

            Graphics::line(
                &mut framebuffer,
                start.x as i32,
                start.y as i32,
                end.x as i32,
                end.y as i32,
                &color::YELLOW,
            );
        }

        return Ok(framebuffer.get_all().clone());
    };

    app.run(&mut update, &render)?;

    Ok(())
}
