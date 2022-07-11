extern crate sdl2;

use std::f32::consts::PI;

use math::round::floor;

use rand::Rng;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::BlendMode;

mod macros;

mod lib;
use crate::lib::color;
use crate::lib::context::{get_application_context, get_application_rendering_context, get_backbuffer};
use crate::lib::draw;
use crate::lib::mesh::get_mesh_from_obj;
use crate::lib::vec::vec2::Vec2;
use crate::lib::vec::vec3::Vec3;

use crate::lib::color::Color;

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

	let mut graphics = get_application_rendering_context(
		app.window
	).unwrap();

	let texture_creator = graphics.canvas.texture_creator();

	let mut backbuffer = get_backbuffer(
		&graphics,
		&texture_creator,
		BlendMode::None
	).unwrap();

	let mut rng = rand::thread_rng();

	// let filepath = "/data/obj/cow.obj";
	// let filepath = "/data/obj/cube.obj";
	// let filepath = "/data/obj/lamp.obj";
	// let filepath = "/data/obj/voxels.obj";
	// let filepath = "/data/obj/voxels2.obj";
	// let filepath = "/data/obj/teapot.obj";
	let filepath = "/data/obj/teapot2.obj";
	// let filepath = "/data/obj/minicooper.obj";
	// let filepath = "/data/obj/minicooper2.obj";
	// let filepath = "/data/obj/jeffrey.obj";
	// let filepath = "/data/obj/jeffrey2.obj";
	// let filepath = "/data/obj/jeffrey3.obj";
	// let filepath = "/data/obj/globe2.obj";
	// let filepath = "/data/obj/pubes.obj";

	let mesh = get_mesh_from_obj(get_absolute_filepath(filepath));

	let mesh_vertices_length = mesh.v.len();
	let mesh_vertex_normals_length = mesh.vn.len();
	let mesh_face_normals_length = mesh.tn.len();

	let height_over_width: f32 = 1.0 / aspect_ratio;

	let z_buffer_size: usize = (screen_width * screen_height) as usize;

	let mut z_buffer: Vec<f32> = Vec::with_capacity(z_buffer_size);

	for _ in 0..z_buffer_size {
		z_buffer.push(f32::MAX);
	}

	let width_scale = screen_width as f32 / 2.0;
	let height_scale = screen_height as f32 / 2.0;

	let mut last_mouse_x: u32 = 0;
	let mut last_mouse_y: u32 = 0;

	let tick_frequency = app.timer.performance_frequency();

	println!("{}", mesh.v[0]);

	let mut world_space_translator = Vec3{

		// default
		x: 0.0,
		y: -1.0,
		z: 10.0,

		// minicooper
		// x: 0.0,
		// y: -10.0,
		// z: 60.0,

	};

	let world_space_scalar = Vec3{
		x: 0.5,
		y: 0.5,
		z: 0.5,
	};

	let mut rotation_radians = Vec3{

		// default
		x: 0.0,
		y: 0.0,
		z: 0.0,

		// minicooper
		// x: PI * -0.5,
		// y: 0.0,
		// z: PI,

	};

	let mut should_render_wireframe = false;
	let mut should_render_shader = true;
	let mut should_render_normals = false;

	let mut frame_start_ticks: u64 = 0;
	let mut frame_end_ticks: u64 = 0;

	let mut light_vector: Vec3 = Vec3{ x: 0.0, y: 0.0, z: 1.0 };
	let mut normalized_light_vector;

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

		for event in app.events.poll_iter() {
			match event {

				Event::Quit { .. } => break 'main,

				Event::KeyDown { keycode: Some(keycode), .. } => {
					match keycode {
						Keycode::Escape { .. } => {
							break 'main
						},
						Keycode::Down|Keycode::S { .. } => {
							world_space_translator.y += 0.1;
						},
						Keycode::Up|Keycode::W { .. } => {
							world_space_translator.y -= 0.1;
						},
						Keycode::Right|Keycode::D { .. } => {
							world_space_translator.x -= 0.1;
						},
						Keycode::Left|Keycode::A { .. } => {
							world_space_translator.x += 0.1;
						},
						Keycode::Q { .. } => {
							world_space_translator.z += 0.1;
						},
						Keycode::E { .. } => {
							world_space_translator.z -= 0.1;
						},
						Keycode::Num1 { .. } => {
							should_render_wireframe = !should_render_wireframe;
						}
						Keycode::Num2 { .. } => {
							should_render_shader = !should_render_shader;
						}
						Keycode::Num3 { .. } => {
							should_render_normals = !should_render_normals;
						}
						_ => {}
					}
				}

				Event::MouseWheel { direction, y, .. } => {
					match direction {
						sdl2::mouse::MouseWheelDirection::Normal {} => {
							world_space_translator.z += (y as f32) / 4.0;
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

		// Translation of vertices to screen space;

		// let last_mouse_x_worldspace = (last_mouse_x as f32 / width_scale) - 1.0;
		// let last_mouse_y_worldspace = -1.0 * ((last_mouse_y as f32 / height_scale) - 1.0);

		// debug_print!("Last mouse position in worldspace: ({}, {})", last_mouse_x_worldspace, last_mouse_y_worldspace);

		rotation_radians.z += 0.25 * PI * delta_t_seconds;
		rotation_radians.z %= 2.0 * PI;

		rotation_radians.x += 0.25 * PI * delta_t_seconds;
		rotation_radians.x %= 2.0 * PI;

		rotation_radians.y += 0.25 * PI * delta_t_seconds;
		rotation_radians.y %= 2.0 * PI;

		// rotation_radians.y = -1.0 * (last_mouse_x as f32) / 100.0;
		// rotation_radians.y %= 2.0 * PI;

		// rotation_radians.x = (last_mouse_y as f32) / 100.0;
		// rotation_radians.x %= 2.0 * PI;

		light_vector.x = -1.0 * (last_mouse_x as f32) / 20.0;
		light_vector.y = (last_mouse_y as f32) / 20.0;

		normalized_light_vector = light_vector.as_normal();

		let mut world_vertices: Vec<Vec3> = vec![Vec3::new(); mesh_vertices_length];

		let mut world_vertex_normals: Vec<Vec3> = vec![];

		if mesh_vertex_normals_length > 0 {
			world_vertex_normals = vec![ Vec3::new(); mesh_vertex_normals_length ];
		}

		for i in 0..mesh_vertices_length {

			world_vertices[i] = mesh.v[i].clone();

			world_vertices[i].rotate_along_z(rotation_radians.z);
			world_vertices[i].rotate_along_x(rotation_radians.x);
			world_vertices[i].rotate_along_y(rotation_radians.y);

			world_vertices[i] *= world_space_scalar;

			world_vertices[i] += world_space_translator;

		}

		for i in 0..mesh_vertex_normals_length {

			world_vertex_normals[i] = mesh.vn[i].clone();

			world_vertex_normals[i].rotate_along_z(rotation_radians.z);
			world_vertex_normals[i].rotate_along_x(rotation_radians.x);
			world_vertex_normals[i].rotate_along_y(rotation_radians.y);

		}

		let mut screen_vertices: Vec<Vec2> = vec![ Vec2::new(); mesh_vertices_length ];

		for i in 0..mesh_vertices_length {

			screen_vertices[i].x = (
				world_vertices[i].x / world_vertices[i].z * height_over_width + 1.0
			) * width_scale;

			screen_vertices[i].y = (
				(-1.0 * world_vertices[i].y) / world_vertices[i].z + 1.0
			) * height_scale;

			screen_vertices[i].z = world_vertices[i].z;

		}

		backbuffer.with_lock(
            None,
            |bytearray, _| {

				let pixel_buffer: &mut draw::PixelBuffer = &mut draw::PixelBuffer{
					pixels: bytemuck::cast_slice_mut(bytearray),
					width: screen_width,
				};

				if should_render_shader {

					for i in 0..z_buffer_size {
						z_buffer[i] = f32::MAX;
					}

				}

				for (index, face) in mesh.f.iter().enumerate() {

					// if index > 2500 {
					// 	continue;
					// }

					// Backface culling

					let screen_vertices = vec![
						screen_vertices[face.0],
						screen_vertices[face.1],
						screen_vertices[face.2],
					];

					let mut world_vertex_normals_for_face: Vec<Vec3> = vec![
						(world_vertices[face.1] - world_vertices[face.0]).cross(world_vertices[face.2] - world_vertices[face.0]),
						(world_vertices[face.2] - world_vertices[face.1]).cross(world_vertices[face.0] - world_vertices[face.1]),
						(world_vertices[face.0] - world_vertices[face.2]).cross(world_vertices[face.1] - world_vertices[face.2]),
					];

					if mesh_face_normals_length > 0 || mesh_vertex_normals_length == mesh_vertices_length {

						if mesh_face_normals_length > 0 {

							let face_normal_indices = mesh.tn[index];

							world_vertex_normals_for_face = vec![
								world_vertex_normals[face_normal_indices.0],
								world_vertex_normals[face_normal_indices.1],
								world_vertex_normals[face_normal_indices.2],
							];

						} else {

							world_vertex_normals_for_face = vec![
								world_vertex_normals[face.0],
								world_vertex_normals[face.1],
								world_vertex_normals[face.2],
							];

						}

					}

					let normalized_face_normal_vector = world_vertex_normals_for_face[0].as_normal();

					let dot_product = normalized_face_normal_vector.dot(world_vertices[face.0].as_normal());

					if dot_product > 0.0 {
						continue;
					}

					if should_render_wireframe {
						draw::poly_line(
							pixel_buffer,
							screen_vertices.as_slice(),
							color::WHITE);
					}

					if should_render_shader {

						// Calculate luminance

						let min_luminance = 150.0;
						let max_luminance = 255.0;

						let light_intensity = 1.0;

						let luminance0 = -1.0 * light_intensity * normalized_light_vector.dot(world_vertex_normals_for_face[0].as_normal());
						let luminance1 = -1.0 * light_intensity * normalized_light_vector.dot(world_vertex_normals_for_face[1].as_normal());
						let luminance2 = -1.0 * light_intensity * normalized_light_vector.dot(world_vertex_normals_for_face[2].as_normal());

						let luminance_avg = (luminance0 + luminance1 + luminance2) / 3.0;

						let scaled_luminance: f32 = min_luminance + luminance_avg * (max_luminance - min_luminance);

						debug_print!("luminance = {}", luminance);

						let color = Color::RGB(
							scaled_luminance as u8,
							scaled_luminance as u8,
							scaled_luminance as u8
							// (0.5 * scaled_luminances) as u8
						);

						draw::triangle_fill(
							pixel_buffer,
							z_buffer.as_mut_slice(),
							screen_width,
							screen_vertices.as_slice(),
							color
						);

					}

					if should_render_normals {

						let world_vertices_for_face = vec![
							world_vertices[face.0],
							world_vertices[face.1],
							world_vertices[face.2],
						];

						let mut world_vertex_normals_for_face: Vec<Vec3> = vec![
							(world_vertices[face.1] - world_vertices[face.0]).cross(world_vertices[face.2] - world_vertices[face.0]),
							(world_vertices[face.2] - world_vertices[face.1]).cross(world_vertices[face.0] - world_vertices[face.1]),
							(world_vertices[face.0] - world_vertices[face.2]).cross(world_vertices[face.1] - world_vertices[face.2]),
						];

						if mesh_face_normals_length > 0 || mesh_vertex_normals_length == mesh_vertices_length {

							if mesh_face_normals_length > 0 {

								let face_normal_indices = mesh.tn[index];

								world_vertex_normals_for_face = vec![
									world_vertex_normals[face_normal_indices.0],
									world_vertex_normals[face_normal_indices.1],
									world_vertex_normals[face_normal_indices.2],
								];

							} else {

								world_vertex_normals_for_face = vec![
									world_vertex_normals[face.0],
									world_vertex_normals[face.1],
									world_vertex_normals[face.2],
								];

							}

						}

						for i in 0..=2 {

							let world_vertex = world_vertices_for_face[i];
							let world_vertex_normal = world_vertex_normals_for_face[i].as_normal() * 0.025;

							// println!("world_vertex: {}", world_vertices_for_face[i]);
							// println!("world_vertex_normal: {}", world_vertex_normals_for_face[i]);

							let world_vertex_relative_normal = world_vertex + world_vertex_normal;

							let screen_vertex_relative_normal = Vec2{
								x: (
									world_vertex_relative_normal.x / world_vertex_relative_normal.z * height_over_width + 1.0
								) * width_scale,
								y: (
									(-1.0 * world_vertex_relative_normal.y) / world_vertex_relative_normal.z + 1.0
								) * height_scale,
								z: 0.0,
							};

							let from_point = screen_vertices[i];

							let to_point = screen_vertex_relative_normal;

							// assert!(
							// 	(from_point.x - to_point.x).abs() < 100.0,
							// 	"Too much space between {} and {}!",
							// 	from_point,
							// 	to_point
							// );

							// draw::set_pixel(
							// 	pixel_buffer,
							// 	to_point.x as u32,
							// 	to_point.y as u32,
							// 	color::RED);

							draw::line(
								pixel_buffer,
							from_point.x as u32,
							from_point.y as u32,
							to_point.x as u32,
							to_point.y as u32,
							color::RED);

						}

					}

				}

				// let screen_light_vector = Vec3{
				// 	x: (
				// 		light_vector.x / light_vector.z * aspect_ratio_how + 1.0
				// 	) * width_scale,
				// 	y: (
				// 		(-1.0 * light_vector.y) / light_vector.z + 1.0
				// 	) * height_scale,
				// 	z: 0.0,
				// };

				// draw::line(
				// 	pixel_buffer,
				// 	0,
				// 	0,
				// 	screen_light_vector.x as u32,
				// 	screen_light_vector.y as u32,
				// 	color::WHITE)

			}
        ).unwrap();

		graphics.canvas.copy(&backbuffer, None, None).unwrap();

		graphics.canvas.present();

		frame_end_ticks = app.timer.performance_counter();

		let delta_ticks = frame_end_ticks - frame_start_ticks;

		let frame_frequency = delta_ticks as f64 / tick_frequency as f64;

		let random: u32 = rng.gen();
		let modulo: u32 = 10;

		if random % modulo == 0 {
			println!("Rendering {} frames per second...", floor(1.0 / frame_frequency, 2));
		}

		// debug_print!("Rendering {} frames per second...", floor(1.0 / frame_frequency, 2));
		// debug_print!("(frame_frequency={}", frame_frequency);

		app.timer.delay(floor(16.666 - frame_frequency, 0) as u32);

	}

	Ok(())

}
