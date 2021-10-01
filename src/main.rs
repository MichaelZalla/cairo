extern crate sdl2;

use math::round::floor;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod macros;

mod draw;
use draw::{PixelBuffer, Color};

mod linear;
use linear::{Vec3, Mesh};

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;
const BYTES_PER_PIXEL: u32 = 4;
const SCREEN_PITCH: u32 = SCREEN_WIDTH * BYTES_PER_PIXEL;
const PIXEL_BUFFER_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * BYTES_PER_PIXEL) as usize;

fn main() -> Result<(), String> {

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
		.opengl()
		// .fullscreen_desktop()
		// .borderless()
		.position_centered()
		.build()
		.unwrap();

	sdl_context.mouse().show_cursor(false);

	let mut canvas = window
        .into_canvas()
        // .accelerated()
		// .present_vsync()
		.build()
		.unwrap();

	let texture_creator = canvas.texture_creator();

	let mut backbuffer = texture_creator
		.create_texture_streaming(
			sdl2::pixels::PixelFormatEnum::RGBA32,
			SCREEN_WIDTH,
			SCREEN_HEIGHT)
		.unwrap();

	backbuffer.update(
		None,
		&vec![0; PIXEL_BUFFER_SIZE],
		SCREEN_PITCH as usize
	).unwrap();

	backbuffer.set_blend_mode(sdl2::render::BlendMode::None);

	let width_scale = SCREEN_WIDTH as f32 / 2.0;
	let height_scale = SCREEN_HEIGHT as f32 / 2.0;

	let white = Color::RGB(255, 255, 255);

	let mut last_mouse_x: u32 = 0;
	let mut last_mouse_y: u32 = 0;

	'main: loop {

		// Main loop

		let frame_start_ms = sdl_context.timer()?.performance_counter();

		// Event polling

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

		// Translation of vertices to screen space;

		let screen_vertices: Vec<Vec3> = mesh.v.clone();

		for mut v in screen_vertices {

			// Scale and translate
			v.x = (v.x + 1.0) * width_scale + width_scale;
			v.y = (v.y + 1.0) * height_scale + height_scale;

			// debug_print!("[x={}, y={}, z={}]", v.x, v.y, v.z);

		}

		canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
		canvas.clear();

		backbuffer.with_lock(
            None,
            |bytearray, _| {

				// let pixel_buffer_local: &mut PixelBuffer = &mut PixelBuffer{
				// 	pixels: bytearray,
				// 	width: SCREEN_WIDTH,
				// 	bytes_per_pixel: BYTES_PER_PIXEL,
				// };

				let mut color = Color::RGB(0,0,0);

				let u32_slice: &mut [u32] = bytemuck::cast_slice_mut(bytearray);

				let mut x: u32 = 0;
				let mut y: u32 = 0;

				for pixel in u32_slice {

					x += 1;

					if x >= SCREEN_WIDTH {
						x = 0;
						y += 1;
					}

					// @NOTE(mzalla) mod is rather expensive here
					// color.r = ((x as i32 - last_mouse_x as i32) % 255) as u8;
					// color.g = ((y as i32 - last_mouse_y as i32) % 255) as u8;

					color.r = (x % 255) as u8;
					color.g = (y % 255) as u8;
					color.b = ((x*y) % 255) as u8;

					let r = (color.r as u32);
					let g = (color.g as u32).rotate_left(8);
					let b = (color.b as u32).rotate_left(16);
					let a = (color.a as u32).rotate_left(24);

					// 00000000111111110000000011111111

					// dbg!(r|g|b|a);

					*pixel = r|g|b|a;

				}

				// for x in 0..SCREEN_WIDTH {
				// 	for y in 0..SCREEN_HEIGHT {

				// 		// draw::set_pixel(
				// 		// 	pixel_buffer_local,
				// 		// 	x,
				// 		// 	y,
				// 		// 	color);

				// 		let pixel_start = y * SCREEN_PITCH + x * BYTES_PER_PIXEL;

				// 		// bytearray[(pixel_start) as usize] = color.r;
				// 		// bytearray[(pixel_start + 1) as usize] = color.g;
				// 		// bytearray[(pixel_start + 2) as usize] = color.b;
				// 		// bytearray[(pixel_start + 3) as usize] = color.a;

				// 	}
				// }

				// draw::line(
				// 	pixel_buffer_local,
				// 	SCREEN_WIDTH / 2,
				// 	SCREEN_HEIGHT / 2,
				// 	last_mouse_x,
				// 	last_mouse_y,
				// 	white);

			}
        ).unwrap();

		canvas.copy(&backbuffer, None, None).unwrap();

		canvas.present();

		let timer = sdl_context.timer()?;
		let frame_end_ms = timer.performance_counter();
		let tick_frequency = timer.performance_frequency();
		let delta_ms = frame_end_ms - frame_start_ms;

		let elapsed = delta_ms as f64 / tick_frequency as f64;

		println!("Rendering {} frames per second...", floor(1.0 / elapsed, 2));

		// timer.delay(floor(16.666 - delta_ms as f64, 0) as u32);

	}

	Ok(())
}
