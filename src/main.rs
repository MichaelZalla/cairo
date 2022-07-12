extern crate sdl2;

use std::cmp::min;

use math::round::floor;

use rand::Rng;

use scenes::mesh_scene::MeshScene;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::BlendMode;

mod macros;

mod lib;
use crate::lib::context::{get_application_context, get_application_rendering_context, get_backbuffer};
use crate::lib::device::{KeyboardState, MouseState};
use crate::lib::graphics::{Graphics, PixelBuffer};

mod scenes;
use crate::lib::scene::Scene;

fn get_absolute_filepath(
	filepath: &str) -> String
{
	let root_directory: String = String::from(env!("CARGO_MANIFEST_DIR"));

	return format!("{}{}", root_directory, filepath).to_string();
}

fn main() -> Result<(), String> {

	let aspect_ratio = 16.0 / 9.0;

	let window_width: u32 = 1200;
	let window_height: u32 = (window_width as f32 / aspect_ratio) as u32;

	let mut app = get_application_context(
		"Cairo (v0.1.0)",
		window_width,
		window_height,
		false,
		false
	).unwrap();

	let screen_width = app.window.size().0;
	let screen_height = app.window.size().1;

	let mut app_rendering_context = get_application_rendering_context(
		app.window
	).unwrap();

	let texture_creator = app_rendering_context.canvas.texture_creator();

	let mut backbuffer =  get_backbuffer(
		&app_rendering_context,
		&texture_creator,
		BlendMode::None,
	).unwrap();

	let graphics = Graphics{
		buffer: PixelBuffer{
			width: screen_width,
			height: screen_height,
			width_over_height: aspect_ratio,
			height_over_width: 1.0 / aspect_ratio,
			pixels: vec![0 as u32; (screen_width * screen_height) as usize],
		},
	};

	let mut scenes = vec![
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/cube.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/cow.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/lamp.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/voxels2.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/teapot.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/teapot2.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/minicooper2.obj")
		),
		MeshScene::new(
			graphics.clone(),
			get_absolute_filepath("/data/obj/jeffrey3.obj")
		),
	];

	let mut current_scene_index = min(0, scenes.len() - 1);

	let tick_frequency = app.timer.performance_frequency();

	// println!("{}", mesh.v[0]);

	let mut frame_start_ticks: u64 = 0;
	let mut frame_end_ticks: u64 = 0;

	let mut rng = rand::thread_rng();

	let mut last_known_mouse_x = 0;
	let mut last_known_mouse_y = 0;

	'main: loop {

		// Main loop

		frame_start_ticks = app.timer.performance_counter();

		if frame_end_ticks == 0 {
			frame_end_ticks = frame_start_ticks;
		}

		let tick_delta = frame_start_ticks - frame_end_ticks;

		let delta_t_seconds = 1.0 / tick_frequency as f32 * tick_delta as f32;

		debug_print!("Slept for {} ticks, {} seconds!", tick_delta, delta_t_seconds);

		// Event polling

		let events = app.events.poll_iter();

		let mut keyboard_state = KeyboardState::new();
		let mut mouse_state = MouseState::new();

		for event in events {
			match event {

				Event::Quit { .. } => break 'main,

				Event::KeyDown { keycode: Some(keycode), .. } => {
					match keycode {
						Keycode::Escape { .. } => {
							break 'main
						},
						Keycode::Num4 { .. } => {
							current_scene_index = min(scenes.len() - 1, 0);
						},
						Keycode::Num5 { .. } => {
							current_scene_index = min(scenes.len() - 1, 1);
						},
						Keycode::Num6 { .. } => {
							current_scene_index = min(scenes.len() - 1, 2);
						},
						Keycode::Num7 { .. } => {
							current_scene_index = min(scenes.len() - 1, 3);
						},
						Keycode::Num8 { .. } => {
							current_scene_index = min(scenes.len() - 1, 4);
						},
						Keycode::Num9 { .. } => {
							current_scene_index = min(scenes.len() - 1, 5);
						},
						Keycode::Num0 { .. } => {
							current_scene_index = min(scenes.len() - 1, 6);
						},
						_ => {
							keyboard_state.keys_pressed.push(keycode);
						}
					}
				}

				Event::MouseMotion { x, y, .. } => {
					last_known_mouse_x = x;
					last_known_mouse_y = y;
				}

				Event::MouseWheel { direction, y, .. } => {
					mouse_state.wheel_did_move = true;
					mouse_state.wheel_direction = direction;
					mouse_state.wheel_y = y;
				}

				_ => {}

			}
		}

		mouse_state.pos.0 = last_known_mouse_x;
		mouse_state.pos.1 = last_known_mouse_y;

		// Update current scene

		scenes[current_scene_index]
			.update(&keyboard_state, &mouse_state, delta_t_seconds);

		backbuffer.with_lock(
            None,
            |write_only_byte_array, _pitch| {

				// Render current scene

				scenes[current_scene_index]
					.render();

				let pixels_as_u8_slice: &[u8] = bytemuck::cast_slice(
					&scenes[current_scene_index].get_pixel_data(),
				);

				let mut index = 0;

				while index < pixels_as_u8_slice.len() {
					write_only_byte_array[index] = pixels_as_u8_slice[index];
					index += 1;
				}

			}
        ).unwrap();

		// Flip buffers

		app_rendering_context.canvas.copy(&backbuffer, None, None).unwrap();

		app_rendering_context.canvas.present();

		frame_end_ticks = app.timer.performance_counter();

		// Report framerate

		let delta_ticks = frame_end_ticks - frame_start_ticks;

		let frame_frequency = delta_ticks as f64 / tick_frequency as f64;

		let random: u32 = rng.gen();
		let modulo: u32 = 10;

		if random % modulo == 0 {
			println!("Rendering {} frames per second...", floor(1.0 / frame_frequency, 2));
		}

		// debug_print!("Rendering {} frames per second...", floor(1.0 / frame_frequency, 2));
		// debug_print!("(frame_frequency={}", frame_frequency);

		// Sleep if we can...

		app.timer.delay(floor(16.666 - frame_frequency, 0) as u32);

	}

	Ok(())

}
