extern crate sdl2;

// #[macro_use]
// extern crate lazy_static;

use std::time::Instant;
// use std::thread::sleep;
// use std::cell::RefCell;
// use std::sync::Mutex;

// use once_cell::sync::Lazy;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
// use sdl2::surface::{SurfaceRef};

mod macros;

mod draw;
use draw::{PixelBuffer, Color};

mod linear;
use linear::{Vec3, Mesh};

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;
const BYTES_PER_PIXEL: u32 = 4;
const PIXEL_BUFFER_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * BYTES_PER_PIXEL) as usize;

// static PIXELS: Lazy<Mutex<[u8; PIXEL_BUFFER_SIZE]>> = Lazy::new(|| {
// 	let mut data: [u8; PIXEL_BUFFER_SIZE] = [0; PIXEL_BUFFER_SIZE];
//     Mutex::new(data);
// });

fn main() -> Result<(), String> {

	let window_rect: Rect = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);

	let mut pixels: [u8; PIXEL_BUFFER_SIZE] = [0; PIXEL_BUFFER_SIZE];
	// let mu: Mutex<[u8; PIXEL_BUFFER_SIZE]> = Mutex::new(pixels);

	let pixel_buffer: &mut PixelBuffer = &mut PixelBuffer{
		// pixels: &mu,
		pixels: &mut pixels,
		width: SCREEN_WIDTH,
		bytes_per_pixel: BYTES_PER_PIXEL,
	};

	let mesh: Mesh = Mesh{
		v: vec![
			Vec3{ x: -0.5, y: 0.5, z: -0.5 }, Vec3{ x: 0.5, y: 0.5, z: -0.5 }, Vec3{ x: -0.5, y: -0.5, z: -0.5 }, Vec3{ x: -0.5, y: -0.5, z: -0.5 },
			Vec3{ x: -0.5, y: 0.5, z:  0.5 }, Vec3{ x: 0.5, y: 0.5, z:  0.5 }, Vec3{ x: -0.5, y: -0.5, z:  0.5 }, Vec3{ x: -0.5, y: -0.5, z:  0.5 },
		],
		f: vec![
			// front faces
			(0,1,2),(1,3,2),
			// top faces
			(0,4,5),(5,1,0),
			// bottom faces
			(2,6,7),(7,3,2),
			// left faces
			(0,4,6),(6,2,0),
			// right faces
			(1,5,7),(7,3,1),
			// back faces
			(4,5,6),(6,5,7),
		]
	};

	let sdl_context = sdl2::init()?;

	let mut events = sdl_context.event_pump()?;

	let video_subsys = sdl_context.video()?;

	let window = video_subsys
		.window("rust-sdl2-pixel", SCREEN_WIDTH, SCREEN_HEIGHT)
		.position_centered()
		.build()
		.unwrap();

	let mut last_mouse_x: u32 = 0;
	let mut last_mouse_y: u32 = 0;

	let width_scale = SCREEN_WIDTH as f32 / 2.0;
	let height_scale = SCREEN_HEIGHT as f32 / 2.0;

	// let black = Color::RGB(0, 0, 0);

	let mut now = Instant::now();
	let mut then: Instant;

	let white = Color::RGB(255, 255, 255);

	'main: loop {

		for event in events.poll_iter() {
			match event {
				Event::Quit { .. } => break 'main,

				Event::KeyDown {
					keycode: Some(keycode),
					..
				} => {
					if keycode == Keycode::Escape {
						break 'main;
					}
					// else if keycode == Keycode::Space {
					// 	debug_print!("space down");
					// 	for i in 0..400 {
					// 		canvas.pixel(i as i16, i as i16, 0xFF000FFu32)?;
					// 	}
					// 	canvas.present();
					// }
				}

				Event::MouseMotion { x, y, .. } => {
					last_mouse_x = x as u32;
					last_mouse_y = y as u32;
				}

				_ => {}
			}
		}

		// Main loop

		// Determine frame-rate

		then = now;
		now = Instant::now();

		let delta_t_duration = now - then;
		let seconds = delta_t_duration.as_secs_f32();
		let milliseconds = (seconds * 1000.0) as u32;

		debug_print!("Slept for {} ms.", (seconds * 1000.0) as u32);
		debug_print!("Rendering {} frames per second...", 1000.0 / milliseconds as f64);

		let mut surface = window.surface(&events)?;

		// Translation of vertices to screen space;

		let screen_vertices: Vec<Vec3> = mesh.v.clone();

		for mut v in screen_vertices {

			// Scale and translate
			v.x = (v.x + 1.0) * width_scale + width_scale;
			v.y = (v.y + 1.0) * height_scale + height_scale;

			// debug_print!("[x={}, y={}, z={}]", v.x, v.y, v.z);

		}

		// for face in mesh.f.iter() {
		// 	let i = face.0;
		// 	let j = face.1;
		// 	let k = face.2;
		// 	let iv = screen_vertices[i];
		// 	let jv = screen_vertices[j];
		// 	let kv = screen_vertices[k];
		// 	draw::line(&canvas, iv.x as i16, iv.y as i16, kv.x as i16, kv.y as i16, white);
		// 	draw::line(&canvas, kv.x as i16, kv.y as i16, jv.x as i16, jv.y as i16, white);
		// 	draw::line(&canvas, jv.x as i16, jv.y as i16, iv.x as i16, iv.y as i16, white);
		// }

		for x in 0..SCREEN_WIDTH {
			for y in 0..SCREEN_HEIGHT {

				let color = Color::RGB(
					((x as i32 - last_mouse_x as i32) % 255) as u8,
					((y as i32 - last_mouse_y as i32) % 255) as u8,
					255,
				);

				draw::set_pixel(
					pixel_buffer,
					x,
					y,
					color);

			}
		}

		draw::line(
			pixel_buffer,
			SCREEN_WIDTH / 2,
			SCREEN_HEIGHT / 2,
			last_mouse_x,
			last_mouse_y,
			white);

		let data_as_surface = sdl2::surface::Surface::from_data(
			pixel_buffer.pixels,
			SCREEN_WIDTH,
			SCREEN_HEIGHT,
			SCREEN_WIDTH * BYTES_PER_PIXEL,
			sdl2::pixels::PixelFormatEnum::RGBA32
		)?;

		let _ = data_as_surface.blit(
			window_rect,
			&mut surface,
			None);

		let _ = surface.finish();

	}

	Ok(())
}
