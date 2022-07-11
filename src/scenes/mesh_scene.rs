use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{lib::{scene::Scene, draw::{self, PixelBuffer}, color::{self, Color}, vec::{vec3::Vec3, vec2::Vec2}, mesh::{Mesh, get_mesh_from_obj}, device::{KeyboardState, MouseState}}, debug_print};

pub struct MeshScene {
	screen_width: u32,
	screen_height: u32,
	width_scale: f32,
	height_scale: f32,
	mesh: Mesh,
	world_space_translator: Vec3,
	world_space_scalar: Vec3,
	rotation_radians: Vec3,
	light_vector: Vec3,
	normalized_light_vector: Vec3,
	z_buffer_size: usize,
	z_buffer: Vec<f32>,
	should_render_wireframe: bool,
	should_render_shader: bool,
	should_render_normals: bool,
}

impl MeshScene {

	pub fn new(screen_width: u32, screen_height: u32, filepath: String) -> Self {

		let mesh = get_mesh_from_obj(filepath);

		let z_buffer_size: usize = (screen_width * screen_height) as usize;

		let mut z_buffer: Vec<f32> = Vec::with_capacity(z_buffer_size);

		for _ in 0..z_buffer_size {
			z_buffer.push(f32::MAX);
		}

		return MeshScene{
			screen_width: screen_width,
			screen_height: screen_height,
			width_scale: screen_width as f32 / 2.0,
			height_scale: screen_height as f32 / 2.0,
			mesh: mesh,
			world_space_translator: Vec3{
				x: 0.0,
				y: -1.0,
				z: 10.0,
			},
			world_space_scalar: Vec3{
				x: 0.5,
				y: 0.5,
				z: 0.5,
			},
			rotation_radians: Vec3{
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			light_vector: Vec3{
				x: 0.0,
				y: 0.0,
				z: 1.0
			},
			normalized_light_vector: Vec3{
				x: 0.0,
				y: 0.0,
				z: 1.0
			},
			z_buffer_size: z_buffer_size,
			z_buffer: z_buffer,
			should_render_wireframe: false,
			should_render_shader: true,
			should_render_normals: false,
		};

	}

}

impl Scene for MeshScene {

	// fn update(&mut self, keyboard_state: KeyboardState, mouse_state: MouseState, delta_t_seconds: f32) -> () {
	fn update(&mut self, keyboard_state: KeyboardState, mouse_state: MouseState, delta_t_seconds: f32) -> () {

		for keycode in keyboard_state.keys_pressed {
			match keycode {
				Keycode::Down|Keycode::S { .. } => {
					self.world_space_translator.y += 0.1;
				},
				Keycode::Up|Keycode::W { .. } => {
					self.world_space_translator.y -= 0.1;
				},
				Keycode::Right|Keycode::D { .. } => {
					self.world_space_translator.x -= 0.1;
				},
				Keycode::Left|Keycode::A { .. } => {
					self.world_space_translator.x += 0.1;
				},
				Keycode::Q { .. } => {
					self.world_space_translator.z += 0.1;
				},
				Keycode::E { .. } => {
					self.world_space_translator.z -= 0.1;
				},
				Keycode::Num1 { .. } => {
					self.should_render_wireframe = !self.should_render_wireframe;
				}
				Keycode::Num2 { .. } => {
					self.should_render_shader = !self.should_render_shader;
				}
				Keycode::Num3 { .. } => {
					self.should_render_normals = !self.should_render_normals;
				}
				_ => {}
			}
		}

		if mouse_state.wheel_did_move {
			match mouse_state.wheel_direction {
				sdl2::mouse::MouseWheelDirection::Normal => {
					self.world_space_translator.z += (mouse_state.wheel_y as f32) / 4.0;
				},
				_ => {}
			}
		}

		self.light_vector.x = -1.0 * (mouse_state.pos.0 as f32) / 20.0;
		self.light_vector.y = (mouse_state.pos.1 as f32) / 20.0;

		self.normalized_light_vector = self.light_vector.as_normal();

		self.rotation_radians.z += 0.25 * PI * delta_t_seconds;
		self.rotation_radians.z %= 2.0 * PI;

		self.rotation_radians.x += 0.25 * PI * delta_t_seconds;
		self.rotation_radians.x %= 2.0 * PI;

		self.rotation_radians.y += 0.25 * PI * delta_t_seconds;
		self.rotation_radians.y %= 2.0 * PI;

	}

	fn render(&mut self, pixel_buffer: &mut PixelBuffer) -> () {

		let mesh_vertices_length = self.mesh.v.len();
		let mesh_vertex_normals_length = self.mesh.vn.len();
		let mesh_face_normals_length = self.mesh.tn.len();

		let aspect_ratio = (self.screen_width as f32) / (self.screen_height as f32);
		let height_over_width: f32 = 1.0 / aspect_ratio;

		// 1. Reset our Z-buffer

		if self.should_render_shader {
			for i in 0..self.z_buffer_size {
				self.z_buffer[i] = f32::MAX;
			}
		}

		// 2. Object-to-world-space transform (vertices)

		let mut world_vertices: Vec<Vec3> = vec![Vec3::new(); mesh_vertices_length];

		for i in 0..mesh_vertices_length {

			world_vertices[i] = self.mesh.v[i].clone();

			world_vertices[i].rotate_along_z(self.rotation_radians.z);
			world_vertices[i].rotate_along_x(self.rotation_radians.x);
			world_vertices[i].rotate_along_y(self.rotation_radians.y);

			world_vertices[i] *= self.world_space_scalar;

			world_vertices[i] += self.world_space_translator;

		}

		// 3. Object-to-world-space transform (normals)

		let mut world_vertex_normals: Vec<Vec3> = vec![];

		if mesh_vertex_normals_length > 0 {
			world_vertex_normals = vec![ Vec3::new(); mesh_vertex_normals_length ];
		}

		for i in 0..mesh_vertex_normals_length {

			world_vertex_normals[i] = self.mesh.vn[i].clone();

			world_vertex_normals[i].rotate_along_z(self.rotation_radians.z);
			world_vertex_normals[i].rotate_along_x(self.rotation_radians.x);
			world_vertex_normals[i].rotate_along_y(self.rotation_radians.y);

		}

		// 4. World-to-screen-space transform (perspective divide)

		let mut screen_vertices: Vec<Vec2> = vec![ Vec2::new(); mesh_vertices_length ];

		for i in 0..mesh_vertices_length {

			screen_vertices[i].x = (
				world_vertices[i].x / world_vertices[i].z * height_over_width + 1.0
			) * self.width_scale;

			screen_vertices[i].y = (
				(-1.0 * world_vertices[i].y) / world_vertices[i].z + 1.0
			) * self.height_scale;

			screen_vertices[i].z = world_vertices[i].z;

		}

		// 5.

		for (index, face) in self.mesh.f.iter().enumerate() {

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

					let face_normal_indices = self.mesh.tn[index];

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

			if self.should_render_wireframe {
				draw::poly_line(
					pixel_buffer,
					screen_vertices.as_slice(),
					color::WHITE);
			}

			if self.should_render_shader {

				// Calculate luminance

				let min_luminance = 150.0;
				let max_luminance = 255.0;

				let light_intensity = 1.0;

				let luminance0 = -1.0 * light_intensity * self.normalized_light_vector.dot(world_vertex_normals_for_face[0].as_normal());
				let luminance1 = -1.0 * light_intensity * self.normalized_light_vector.dot(world_vertex_normals_for_face[1].as_normal());
				let luminance2 = -1.0 * light_intensity * self.normalized_light_vector.dot(world_vertex_normals_for_face[2].as_normal());

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
					self.z_buffer.as_mut_slice(),
					self.screen_width,
					screen_vertices.as_slice(),
					color
				);

			}

			if self.should_render_normals {

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

						let face_normal_indices = self.mesh.tn[index];

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
						) * self.width_scale,
						y: (
							(-1.0 * world_vertex_relative_normal.y) / world_vertex_relative_normal.z + 1.0
						) * self.height_scale,
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

}