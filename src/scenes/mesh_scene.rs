use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{lib::{scene::Scene, color::{self, Color}, vec::{vec3::Vec3, vec2::Vec2}, mesh::{Mesh, get_mesh_from_obj, Face}, device::{KeyboardState, MouseState}, graphics::Graphics, pipeline::Pipeline}};

type Triangle<T> = Vec<T>;

#[derive(Copy, Clone, Default)]
struct Vertex {
	p: Vec3,
	n: Vec3,
}

impl Vertex {

	pub fn new() -> Self {
		Default::default()
	}

}

pub struct MeshScene {
	pipeline: Pipeline,
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

	pub fn new(
		graphics: Graphics,
		filepath: String) -> Self
	{

		let mesh = get_mesh_from_obj(filepath);

		let z_buffer_size: usize = (graphics.buffer.width * graphics.buffer.height) as usize;

		let mut z_buffer: Vec<f32> = Vec::with_capacity(z_buffer_size);

		for _ in 0..z_buffer_size {
			z_buffer.push(f32::MAX);
		}

		let width_scale = graphics.buffer.width as f32 / 2.0;
		let height_scale = graphics.buffer.height as f32 / 2.0;

		return MeshScene{
			pipeline: Pipeline{
				graphics: graphics,
			},
			width_scale: width_scale,
			height_scale: height_scale,
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
			should_render_wireframe: true,
			should_render_shader: true,
			should_render_normals: false,
		};

	}

	fn process_world_vertices(
		&mut self) -> ()
	{

		let mesh_v_len = self.mesh.v.len();

		let mut world_vertices: Vec<Vertex> = vec![Vertex::new(); mesh_v_len];

		// Object-to-world-space transform

		for i in 0..mesh_v_len {

			world_vertices[i].p = self.mesh.v[i].clone();

			world_vertices[i].p.rotate_along_z(self.rotation_radians.z);
			world_vertices[i].p.rotate_along_x(self.rotation_radians.x);
			world_vertices[i].p.rotate_along_y(self.rotation_radians.y);

			world_vertices[i].p *= self.world_space_scalar;

			world_vertices[i].p += self.world_space_translator;

		}

		let faces = self.mesh.f.clone();

		self.process_triangles(faces, world_vertices);

	}

	fn process_triangles(
		&mut self,
		faces: Vec<Face>,
		vertices: Vec<Vertex>) -> ()
	{

		let mut triangles: Vec<Triangle<Vertex>> = vec![];

		let mesh_v_len = self.mesh.v.len();
		let mesh_vn_len = self.mesh.vn.len();
		let mesh_tn_len = self.mesh.tn.len();

		for (face_index, face) in faces.iter().enumerate() {

			// Resolve normals for current triangle;

			let mut world_vertex_normals_for_face: Vec<Vec3> = vec![];

			if mesh_tn_len > 0 {
				world_vertex_normals_for_face.push(self.mesh.vn[self.mesh.tn[face_index].0].clone());
				world_vertex_normals_for_face.push(self.mesh.vn[self.mesh.tn[face_index].1].clone());
				world_vertex_normals_for_face.push(self.mesh.vn[self.mesh.tn[face_index].2].clone());
			}
			else if mesh_vn_len == mesh_v_len {
				world_vertex_normals_for_face.push(self.mesh.vn[face.0].clone());
				world_vertex_normals_for_face.push(self.mesh.vn[face.1].clone());
				world_vertex_normals_for_face.push(self.mesh.vn[face.2].clone());
			}
			else {
				world_vertex_normals_for_face.push(
					(vertices[face.1].p - vertices[face.0].p).cross(vertices[face.2].p - vertices[face.0].p)
				);
				world_vertex_normals_for_face.push(
					(vertices[face.2].p - vertices[face.1].p).cross(vertices[face.0].p - vertices[face.1].p)
				);
				world_vertex_normals_for_face.push(
					(vertices[face.0].p - vertices[face.2].p).cross(vertices[face.1].p - vertices[face.2].p)
				);
			}

			// Rotate normals

			if mesh_tn_len > 0 || mesh_vn_len == mesh_v_len {

				world_vertex_normals_for_face[0].rotate_along_z(self.rotation_radians.z);
				world_vertex_normals_for_face[0].rotate_along_x(self.rotation_radians.x);
				world_vertex_normals_for_face[0].rotate_along_y(self.rotation_radians.y);

				world_vertex_normals_for_face[1].rotate_along_z(self.rotation_radians.z);
				world_vertex_normals_for_face[1].rotate_along_x(self.rotation_radians.x);
				world_vertex_normals_for_face[1].rotate_along_y(self.rotation_radians.y);

				world_vertex_normals_for_face[2].rotate_along_z(self.rotation_radians.z);
				world_vertex_normals_for_face[2].rotate_along_x(self.rotation_radians.x);
				world_vertex_normals_for_face[2].rotate_along_y(self.rotation_radians.y);

			}

			world_vertex_normals_for_face[0] = world_vertex_normals_for_face[0].as_normal();
			world_vertex_normals_for_face[1] = world_vertex_normals_for_face[1].as_normal();
			world_vertex_normals_for_face[2] = world_vertex_normals_for_face[2].as_normal();

			// Cull backfaces

			let dot_product = world_vertex_normals_for_face[0].dot(vertices[face.0].p.as_normal());

			if dot_product > 0.0 {
				continue;
			}

			triangles.push(vec![
				Vertex{
					p: vertices[face.0].p,
					n: world_vertex_normals_for_face[0],
				},
				Vertex{
					p: vertices[face.1].p,
					n: world_vertex_normals_for_face[1],
				},
				Vertex{
					p: vertices[face.2].p,
					n: world_vertex_normals_for_face[2],
				},
			]);

		}

		for triangle in triangles {
			self.process_triangle(triangle);
		}

	}

	fn process_triangle(
		&mut self,
		triangle: Triangle<Vertex>) -> ()
	{
		// @TODO(mzalla) Geometry shader?

		self.post_process_triangle_vertices(triangle);
	}

	fn post_process_triangle_vertices(
		&mut self,
		triangle: Triangle<Vertex>) -> ()
	{

		let mut points: Vec<Vec2> = vec![
			Vec2{
				x: triangle[0].p.x,
				y: triangle[0].p.y,
				z: triangle[0].p.z,
			},
			Vec2{
				x: triangle[1].p.x,
				y: triangle[1].p.y,
				z: triangle[1].p.z,
			},
			Vec2{
				x: triangle[2].p.x,
				y: triangle[2].p.y,
				z: triangle[2].p.z,
			},
		];

		// Screen-space perspective divide

		for mut point in points.as_mut_slice() {

			point.x = (
				point.x / point.z * self.pipeline.graphics.buffer.height_over_width + 1.0
			) * self.width_scale;

			point.y = (
				(-1.0 * point.y) / point.z + 1.0
			) * self.height_scale;

		}

		if self.should_render_wireframe {
			self.pipeline.graphics.poly_line(points.as_slice(), color::WHITE);
		}

		// Interpolate entire Vertex (all attributes) when drawing (scanline
		// interpolant)

		if self.should_render_shader {

			// Calculate luminance

			let min_luminance = 150.0;
			let max_luminance = 255.0;

			let light_intensity = 1.0;

			let luminance0 = -1.0 * light_intensity * self.normalized_light_vector.dot(triangle[0].n);
			let luminance1 = -1.0 * light_intensity * self.normalized_light_vector.dot(triangle[1].n);
			let luminance2 = -1.0 * light_intensity * self.normalized_light_vector.dot(triangle[2].n);

			let luminance_avg = (luminance0 + luminance1 + luminance2) / 3.0;

			let scaled_luminance: f32 = min_luminance + luminance_avg * (max_luminance - min_luminance);

			// println!("luminance_avg = {}", luminance_avg);

			let color = Color::RGB(
				scaled_luminance as u8,
				scaled_luminance as u8,
				scaled_luminance as u8
				// (0.5 * scaled_luminances) as u8
			);

			let z_buffer_width = self.pipeline.graphics.buffer.width;

			self.pipeline.graphics.triangle_fill(
				self.z_buffer.as_mut_slice(),
				z_buffer_width,
				points.as_slice(),
				color
			);

		}

		if self.should_render_normals {

			for i in 0..=2 {

				let world_vertex_relative_normal = triangle[i].p + (triangle[i].n * 0.05);

				let screen_vertex_relative_normal = Vec2{
					x: (
						world_vertex_relative_normal.x / world_vertex_relative_normal.z * self.pipeline.graphics.buffer.height_over_width + 1.0
					) * self.width_scale,
					y: (
						(-1.0 * world_vertex_relative_normal.y) / world_vertex_relative_normal.z + 1.0
					) * self.height_scale,
					z: 0.0,
				};

				let from_point = points[i];

				let to_point = screen_vertex_relative_normal;

				self.pipeline.graphics.line(
					from_point.x as u32,
				from_point.y as u32,
				to_point.x as u32,
				to_point.y as u32,
				color::RED);

			}

		}

	}

}

impl Scene for MeshScene {

	fn update(&mut self, keyboard_state: &KeyboardState, mouse_state: &MouseState, delta_t_seconds: f32) -> () {

		for keycode in &keyboard_state.keys_pressed {
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

	fn render(&mut self) -> () {

		self.pipeline.graphics.buffer.clear();

		if self.should_render_shader {
			for i in 0..self.z_buffer_size {
				self.z_buffer[i] = f32::MAX;
			}
		}

		self.process_world_vertices();

	}

	fn get_pixel_data(&self) -> &Vec<u32> {
		return self.pipeline.graphics.get_pixel_data();
	}

}