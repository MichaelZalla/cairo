extern crate sdl2;

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
use crate::lib::draw::PixelBuffer;
use crate::lib::graphics::Graphics;

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

	let mut graphics = Graphics{
		texture: get_backbuffer(
			&app_rendering_context,
			&texture_creator,
			BlendMode::None,
		).unwrap(),
	};

	// let filepath = "/data/obj/cow.obj";
	// let filepath = "/data/obj/cube.obj";
	// let filepath = "/data/obj/lamp.obj";
	// let filepath = "/data/obj/voxels.obj";
	// let filepath = "/data/obj/voxels2.obj";
	// let filepath = "/data/obj/teapot.obj";
	// let filepath = "/data/obj/teapot2.obj";
	// let filepath = "/data/obj/minicooper.obj";
	// let filepath = "/data/obj/minicooper2.obj";
	// let filepath = "/data/obj/jeffrey.obj";
	// let filepath = "/data/obj/jeffrey2.obj";
	// let filepath = "/data/obj/jeffrey3.obj";
	// let filepath = "/data/obj/globe2.obj";
	// let filepath = "/data/obj/pubes.obj";

	let mut scenes: Vec<MeshScene> = vec![
		MeshScene::new(
			screen_width,
			screen_height,
			get_absolute_filepath("/data/obj/voxels2.obj")
		),
		// MeshScene::new(
		// 	screen_width,
		// 	screen_height,
		// 	get_absolute_filepath("/data/obj/minicooper2.obj")
		// )
	];

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

		// 1b. Scene update (rotation, velocity, etc)

		for scene in scenes.as_mut_slice() {
			scene.update(&keyboard_state, &mouse_state, delta_t_seconds);
		}

		// Indexes triangle list

		// 0. Split vertices and indices
		// 1a. Vertex stream -> Vertex transformer
		// 1b. Index stream
		// 2. Triangle assembler (vertices + indices -> Triangle[])
		//  - Backface culling
		// 3. World-space-to-screen-space transformer
		// 4. Triange rasterizer
		// 5. Pixel shader
		// 6. PutPixel

		// Scene::Update(keyboardState, mouseState, deltaT)

		// pub struct Triangle<T = Vec3> {
		// 	pub v0: T,
		// 	pub v1: T,
		// 	pub v2: T,
		// }

		// Interpolate entire Vertex (all attributes) when drawing (scanline
		// interpolant)

		graphics.texture.with_lock(
            None,
            |write_only_byte_array, _pitch| {

				let pixels: &mut [u32] = bytemuck::cast_slice_mut(write_only_byte_array);

				let mut pixel_buffer = PixelBuffer{
					pixels: pixels,
					width: screen_width,
				};

				for scene in scenes.as_mut_slice() {
					scene.render(&mut pixel_buffer);
				}

			}
        ).unwrap();

		// Page-flip

		app_rendering_context.canvas.copy(&graphics.texture, None, None).unwrap();

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
