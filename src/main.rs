extern crate sdl2;

use math::round::floor;

use sdl2::{event::Event, mouse::MouseWheelDirection};
use sdl2::keyboard::Keycode;

mod macros;

mod draw;
use draw::{PixelBuffer, Color};

mod linear;
use linear::{Vec2, Vec3, Mesh};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const ASPECT_RATIO: f32 = SCREEN_HEIGHT as f32 / SCREEN_WIDTH as f32;
const BYTES_PER_PIXEL: u32 = 4;
const SCREEN_PITCH: u32 = SCREEN_WIDTH * BYTES_PER_PIXEL;
const PIXEL_BUFFER_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * BYTES_PER_PIXEL) as usize;

fn main() -> Result<(), String> {

	let mesh: &Mesh = &Mesh{
		v: vec![
			Vec3{ x: -0.5, y: 0.5, z: -0.5 }, Vec3{ x: 0.5, y: 0.5, z: -0.5 }, Vec3{ x: -0.5, y: -0.5, z: -0.5 }, Vec3{ x: 0.5, y: -0.5, z: -0.5 },
			Vec3{ x: -0.5, y: 0.5, z:  0.5 }, Vec3{ x: 0.5, y: 0.5, z:  0.5 }, Vec3{ x: -0.5, y: -0.5, z:  0.5 }, Vec3{ x: 0.5, y: -0.5, z:  0.5 },
		],
		f: vec![
			// front faces
			(0,1,3),(0,2,3),
			// top faces
			(0,4,5),(0,1,5),
			// bottom faces
			(2,6,7),(2,3,7),
			// left faces
			(0,4,6),(0,2,6),
			// right faces
			(1,5,7),(1,3,7),
			// back faces
			(4,5,7),(4,6,7),
		]
	};

	let mesh_vertices_length = mesh.v.len();

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

	let mut timer = sdl_context.timer()?;
	let tick_frequency = timer.performance_frequency();

	let mut world_space_translator = Vec3{
		x: 0.0,
		y: 0.0,
		z: 1.0,
	};

	let mut world_space_scalar = Vec3{
		x: 1.0,
		y: 1.0,
		z: 1.0,
	};

	'main: loop {

		// Main loop

		let frame_start_ticks = timer.performance_counter();

		// Event polling

		for event in events.poll_iter() {
			match event {

				Event::Quit { .. } => break 'main,

				Event::KeyDown {
					keycode: Some(keycode),
					..
				} => {

					match keycode {

						Keycode::Escape { .. } => break 'main,

						Keycode::Up { .. } => {
							if last_mouse_y >= 25 {
								last_mouse_y -= 25;
							}
						},
						Keycode::Down { .. } => {
							if last_mouse_y < (SCREEN_HEIGHT - 25) {
								last_mouse_y += 25;
							}
						},
						Keycode::Left { .. } => {
							if last_mouse_x >= 25 {
								last_mouse_x -= 25;
							}
						},
						Keycode::Right { .. } => {
							if last_mouse_x < (SCREEN_WIDTH - 25) {
								last_mouse_x += 25;
							}
						},

						_ => {}

					}
				}

				Event::MouseWheel {
					direction,
					which,
					x,
					y,
					..
				} => {

					match direction {

						sdl2::mouse::MouseWheelDirection::Normal {} => {
							world_space_translator.z = world_space_translator.z + (y as f32) / 4.0;
						}

						_ => {}

					}

				}

				Event::MouseMotion { x, y, .. } => {
					last_mouse_x = x as u32;
					last_mouse_y = y as u32;
				}

				_ => {}

			}
		}

		// Translation of vertices to screen space;



		let last_mouse_x_worldspace = (last_mouse_x as f32 / width_scale) - 1.0;
		let last_mouse_y_worldspace = -1.0 * ((last_mouse_y as f32 / height_scale) - 1.0);

		let last_mouse_world_space_translator = Vec3{
			x: last_mouse_x_worldspace,
			y: last_mouse_y_worldspace,
			z: 1.0,
		};

		// let worldspace_to_screenspace_translator: Vec2 = Vec2 {
		// 	x: width_scale + last_mouse_x_worldspace,
		// 	y: height_scale + last_mouse_y_worldspace,
		// };

		let mut world_vertices: Vec<Vec3> = vec![ Vec3{ x: 0.0, y: 0.0, z: 0.0 }; mesh_vertices_length ];

		for i in 0..mesh_vertices_length {

			world_vertices[i] = mesh.v[i].clone();

			world_vertices[i] = world_vertices[i] * world_space_scalar;

			world_vertices[i] = world_vertices[i] + world_space_translator;
			world_vertices[i] = world_vertices[i] + last_mouse_world_space_translator;

		}

		let mut screen_vertices: Vec<Vec2> = vec![ Vec2{ x: 0.0, y: 0.0 }; mesh_vertices_length ];

		for i in 0..mesh_vertices_length {

			// Scale and translate

			screen_vertices[i].x = (
				world_vertices[i].x / (world_vertices[i].z) * ASPECT_RATIO + 1.0
			) * width_scale;

			screen_vertices[i].y = (
				(-1.0 * world_vertices[i].y) / (world_vertices[i].z) + 1.0
			) * height_scale;

			// debug_print!("screen_vertices[{}] = ({}, {})", i, screen_vertices[i].x, screen_vertices[i].y);

		}

		canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));

		canvas.clear();

		backbuffer.with_lock(
            None,
            |bytearray, _| {

				let pixel_buffer: &mut PixelBuffer = &mut PixelBuffer{
					pixels: bytemuck::cast_slice_mut(bytearray),
					width: SCREEN_WIDTH,
					// bytes_per_pixel: BYTES_PER_PIXEL,
				};

				for face in &mesh.f {

					let c = draw::Color::RGB(
						255,255,255
						// (screen_vertices[face.0].x % u8::MAX as f32) as u8,
						// (screen_vertices[face.1].x % u8::MAX as f32) as u8,
						// (screen_vertices[face.2].x % u8::MAX as f32) as u8,
					);

					// let a = Vec2{x: 5.0, y: 5.0};
					// let b = Vec2{x: 10.0, y: 12.0};

					// println!("a + b = {}", a + b);
					// println!("b + a = {}", b + a);
					// println!("a - b = {}", a - b);
					// println!("b - a = {}", b - a);
					// println!("a * b = {}", a * b);
					// println!("b * a = {}", b * a);
					// println!("a / b = {}", a / b);
					// println!("b / a = {}", b / a);
					// println!("a * 7 = {}", a * 7.0);
					// println!("a / 2 = {}", a / 7.0);

					draw::poly(
						pixel_buffer,
						vec![
							screen_vertices[face.0],
							screen_vertices[face.1],
							screen_vertices[face.2],
						].as_slice(),
						c);

				}

			}
        ).unwrap();

		canvas.copy(&backbuffer, None, None).unwrap();

		canvas.present();

		let frame_end_ticks = timer.performance_counter();

		let delta_ticks = frame_end_ticks - frame_start_ticks;

		let frame_frequency = delta_ticks as f64 / tick_frequency as f64;

		// debug_print!("Rendering {} frames per second...", floor(1.0 / frame_frequency, 2));
		// debug_print!("(frame_frequency={}", frame_frequency);

		timer.delay(floor(16.666 - delta_ticks as f64, 0) as u32);

	}

	Ok(())
}
