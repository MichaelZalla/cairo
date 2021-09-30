extern crate sdl2;

use sdl2::event::Event;
use sdl2::image;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color,PixelFormatEnum};
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::{Window};

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

mod draw;

#[derive(Debug, Copy, Clone)]
struct Vec3 {
	x: f32,
	y: f32,
	z: f32,
}

struct Mesh {
	v: Vec<Vec3>,
	f: Vec<(usize, usize, usize)>,
}

fn main() -> Result<(), String> {

	// let vertices: Vec<Vec3> = ;

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
		// .opengl()
		.build()
		.unwrap();

	let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

	// let texture_creator = canvas.texture_creator();

	// https://rust-sdl2.github.io/rust-sdl2/sdl2/video/struct.Window.html#method.surface
	// let mut surface = window.surface(&events)?;

	// ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

	// let surface = (window);
	// let surface = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::RGB24).unwrap();
	// let texture = texture_creator.create_texture_from_surface(surface).unwrap();

	canvas.clear();
	canvas.present();

	let mut last_mouse_x: i16 = 0;
	let mut last_mouse_y: i16 = 0;

	let width_scale = SCREEN_WIDTH as f32 / 2.0;
	let height_scale = SCREEN_HEIGHT as f32 / 2.0;

	let black = Color::RGB(0, 0, 0);
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
					// 	println!("space down");
					// 	for i in 0..400 {
					// 		canvas.pixel(i as i16, i as i16, 0xFF000FFu32)?;
					// 	}
					// 	canvas.present();
					// }
				}

				Event::MouseMotion { x, y, .. } => {
					last_mouse_x = x as i16;
					last_mouse_y = y as i16;
				}

				_ => {}
			}
		}

		// Main loop

		canvas.set_draw_color(black);
		canvas.clear();

		// Translation of vertices to screen space;

		let screen_vertices: Vec<Vec3> = mesh.v.clone();

		for mut v in screen_vertices {

			// Scale and translate
			v.x = (v.x + 1.0) * width_scale + width_scale;
			v.y = (v.y + 1.0) * height_scale + height_scale;

			println!("[x={}, y={}, z={}]", v.x, v.y, v.z);

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

		draw::line(&canvas, SCREEN_WIDTH as i16 / 2, SCREEN_HEIGHT as i16 / 2, last_mouse_x, last_mouse_y, white);

		canvas.present();

	}

	Ok(())
}
