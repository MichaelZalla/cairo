use crate::vertices::default_vertex::DefaultVertex;

use super::{graphics::Graphics, vec::{vec3::Vec3, vec2::Vec2}, mesh::Mesh, color::{self, Color}, effect::Effect};

#[derive(Copy, Clone, Default)]
struct Triangle<T> {
	v0: T,
	v1: T,
	v2: T,
}

#[derive(Copy, Clone, Default)]
pub struct PipelineOptions {
	pub should_render_wireframe: bool,
	pub should_render_shader: bool,
	pub should_render_normals: bool,
}

type Vertex = DefaultVertex;

pub struct Pipeline<T: Effect> {
	options: PipelineOptions,
	graphics: Graphics,
	buffer_width_over_2: f32,
	buffer_height_over_2: f32,
	scale: Vec3,
	rotation: Vec3,
	translation: Vec3,
	directional_light_normal: Vec3,
	z_buffer: Vec<f32>,
	effect: T,
}

impl<T: Effect<Vertex = DefaultVertex>> Pipeline<T> where T: Effect {

	pub fn new(
		graphics: Graphics,
		effect: T,
		options: PipelineOptions) -> Self
	{

		let z_buffer_size: usize = (graphics.buffer.width * graphics.buffer.height) as usize;

		let mut z_buffer: Vec<f32> = Vec::with_capacity(z_buffer_size);

		for _ in 0..z_buffer_size {
			z_buffer.push(f32::MAX);
		}

		let buffer_width_over_2 = (graphics.buffer.width as f32) / 2.0;
		let buffer_height_over_2 = (graphics.buffer.height as f32) / 2.0;

		return Pipeline{
			options: options,
			graphics: graphics,
			buffer_width_over_2: buffer_width_over_2,
			buffer_height_over_2: buffer_height_over_2,
			scale: Vec3::new(),
			rotation: Vec3::new(),
			translation: Vec3::new(),
			directional_light_normal: Vec3{
				x: 0.0,
				y: 0.0,
				z: 1.0
			},
			z_buffer: z_buffer,
			effect: effect,
		};

	}

	pub fn get_pixel_data(
		&self) -> &Vec<u32>
	{
		return self.graphics.get_pixel_data();
	}

	pub fn set_options(
		&mut self,
		options: PipelineOptions) -> ()
	{
		self.options = options;
	}

	pub fn set_scale(
		&mut self,
		matrix: Vec3) -> ()
	{
		self.scale = matrix;
	}

	pub fn set_rotation(
		&mut self,
		matrix: Vec3) -> ()
	{
		self.rotation = matrix;
	}

	pub fn set_translation(
		&mut self,
		matrix: Vec3) -> ()
	{
		self.translation = matrix;
	}

	pub fn set_light_normal(
		&mut self,
		normal: Vec3) -> ()
	{
		self.directional_light_normal = normal;
	}

	pub fn render(
		&mut self,
		mesh: &Mesh) -> ()
	{

		self.graphics.buffer.clear();

		if self.options.should_render_shader {
			for i in 0..self.z_buffer.len() {
				self.z_buffer[i] = f32::MAX;
			}
		}

		self.process_world_vertices(mesh);

	}

	fn process_world_vertices(
		&mut self,
		mesh: &Mesh) -> ()
	{

		let mesh_v_len = mesh.v.len();

		let mut world_vertices: Vec<Vertex> = vec![Vertex::new(); mesh_v_len];

		// Object-to-world-space transform

		for i in 0..mesh_v_len {

			world_vertices[i].p = mesh.v[i].clone();

			world_vertices[i].p.rotate_along_z(self.rotation.z);
			world_vertices[i].p.rotate_along_x(self.rotation.x);
			world_vertices[i].p.rotate_along_y(self.rotation.y);

			world_vertices[i].p *= self.scale;

			world_vertices[i].p += self.translation;

		}

		self.process_triangles(mesh, world_vertices);

	}

	fn process_triangles(
		&mut self,
		mesh: &Mesh,
		vertices: Vec<Vertex>) -> ()
	{

		let faces = mesh.f.clone();

		let mut triangles: Vec<Triangle<Vertex>> = vec![];

		let mesh_v_len = mesh.v.len();
		let mesh_vn_len = mesh.vn.len();
		let mesh_tn_len = mesh.tn.len();

		for (face_index, face) in faces.iter().enumerate() {

			// Resolve normals for current triangle;

			let mut world_vertex_normals_for_face: Vec<Vec3> = vec![];

			if mesh_tn_len > 0 {
				world_vertex_normals_for_face.push(mesh.vn[mesh.tn[face_index].0].clone());
				world_vertex_normals_for_face.push(mesh.vn[mesh.tn[face_index].1].clone());
				world_vertex_normals_for_face.push(mesh.vn[mesh.tn[face_index].2].clone());
			}
			else if mesh_vn_len == mesh_v_len {
				world_vertex_normals_for_face.push(mesh.vn[face.0].clone());
				world_vertex_normals_for_face.push(mesh.vn[face.1].clone());
				world_vertex_normals_for_face.push(mesh.vn[face.2].clone());
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

				world_vertex_normals_for_face[0].rotate_along_z(self.rotation.z);
				world_vertex_normals_for_face[0].rotate_along_x(self.rotation.x);
				world_vertex_normals_for_face[0].rotate_along_y(self.rotation.y);

				world_vertex_normals_for_face[1].rotate_along_z(self.rotation.z);
				world_vertex_normals_for_face[1].rotate_along_x(self.rotation.x);
				world_vertex_normals_for_face[1].rotate_along_y(self.rotation.y);

				world_vertex_normals_for_face[2].rotate_along_z(self.rotation.z);
				world_vertex_normals_for_face[2].rotate_along_x(self.rotation.x);
				world_vertex_normals_for_face[2].rotate_along_y(self.rotation.y);

			}

			world_vertex_normals_for_face[0] = world_vertex_normals_for_face[0].as_normal();
			world_vertex_normals_for_face[1] = world_vertex_normals_for_face[1].as_normal();
			world_vertex_normals_for_face[2] = world_vertex_normals_for_face[2].as_normal();

			// Cull backfaces

			let dot_product = world_vertex_normals_for_face[0].dot(vertices[face.0].p.as_normal());

			if dot_product > 0.0 {
				continue;
			}

			triangles.push(Triangle{
				v0: Vertex{
					p: vertices[face.0].p,
					n: world_vertex_normals_for_face[0],
				},
				v1: Vertex{
					p: vertices[face.1].p,
					n: world_vertex_normals_for_face[1],
				},
				v2: Vertex{
					p: vertices[face.2].p,
					n: world_vertex_normals_for_face[2],
				},
			});

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

		let world_vertex_relative_normals = [
			triangle.v0.p + (triangle.v0.n * 0.05),
			triangle.v1.p + (triangle.v1.n * 0.05),
			triangle.v2.p + (triangle.v2.n * 0.05),
		];

		// Screen-space perspective divide

		let screen_v0 = Vertex{
			p: Vec3{
				x: (
					triangle.v0.p.x / triangle.v0.p.z * self.graphics.buffer.height_over_width + 1.0
				) * self.buffer_width_over_2,
				y: (
					(-1.0 * triangle.v0.p.y) / triangle.v0.p.z + 1.0
				) * self.buffer_height_over_2,
				z: triangle.v0.p.z,
			},
			n: triangle.v0.n.clone(),
		};

		let screen_v1 = Vertex{
			p: Vec3{
				x: (
					triangle.v1.p.x / triangle.v1.p.z * self.graphics.buffer.height_over_width + 1.0
				) * self.buffer_width_over_2,
				y: (
					(-1.0 * triangle.v1.p.y) / triangle.v1.p.z + 1.0
				) * self.buffer_height_over_2,
				z: triangle.v1.p.z,
			},
			n: triangle.v1.n.clone(),
		};

		let screen_v2 = Vertex{
			p: Vec3{
				x: (
					triangle.v2.p.x / triangle.v2.p.z * self.graphics.buffer.height_over_width + 1.0
				) * self.buffer_width_over_2,
				y: (
					(-1.0 * triangle.v2.p.y) / triangle.v2.p.z + 1.0
				) * self.buffer_height_over_2,
				z: triangle.v2.p.z,
			},
			n: triangle.v2.n.clone(),
		};

		let screen_vertices = [screen_v0, screen_v1, screen_v2];

		if self.options.should_render_wireframe {

			let mut points: Vec<Vec2> = vec![];

			for v in screen_vertices {
				points.push(Vec2 {
					x: v.p.x,
					y: v.p.y,
					z: v.p.z,
				});
			}

			self.graphics.poly_line(
				points.as_slice(),
				color::WHITE
			);
		}

		// Interpolate entire vertex (all attributes) when drawing (scanline
		// interpolant)

		if self.options.should_render_shader {
			self.triangle_fill(screen_v0, screen_v1, screen_v2);
		}

		if self.options.should_render_normals {

			for (index, v) in screen_vertices.iter().enumerate() {

				let world_vertex_relative_normal = world_vertex_relative_normals[index];

				let screen_vertex_relative_normal = Vec2{
					x: (
						world_vertex_relative_normal.x / world_vertex_relative_normal.z * self.graphics.buffer.height_over_width + 1.0
					) * self.buffer_width_over_2,
					y: (
						(-1.0 * world_vertex_relative_normal.y) / world_vertex_relative_normal.z + 1.0
					) * self.buffer_height_over_2,
					z: 0.0,
				};

				let from = v.p;
				let to = screen_vertex_relative_normal;

				self.graphics.line(from.x as u32, from.y as u32, to.x as u32, to.y as u32, color::RED);

			}

		}

	}

	#[inline(always)]
	fn set_pixel(
		&mut self,
		x: u32,
		y: u32,
		z: f32,
		color: color::Color) -> ()
	{

		if x > (self.graphics.buffer.width - 1) || y > (self.graphics.buffer.pixels.len() as u32 / self.graphics.buffer.width as u32 - 1) {
			// Prevents panic! inside of self.graphics.set_pixel();
			return;
		}

		let z_buffer_index = (y * self.graphics.buffer.width + x) as usize;

		if z_buffer_index >= self.z_buffer.len() {
			panic!("Call to draw::set_pixel with invalid coordinate ({},{})!", x, y);
		}

		if z < self.z_buffer[z_buffer_index] {
			self.z_buffer[z_buffer_index] = z;
		} else {
			return;
		}

		self.graphics.set_pixel(x, y, color);

	}

	#[inline(always)]
	fn flat_top_triangle_fill(
		&mut self,
		v0: Vertex,
		v1: Vertex,
		v2: Vertex) -> ()
	{

		// @NOTE(mzalla) v0 as top-left vertex
		// @NOTE(mzalla) v1 as top-right vertex
		// @NOTE(mzalla) v2 as bottom vertex

		let left_step_x = (v2.p.x - v0.p.x) / (v2.p.y - v0.p.y);
		let right_step_x = (v2.p.x - v1.p.x) / (v2.p.y - v1.p.y);

		let left_step_z = (v2.p.z - v0.p.z) / (v2.p.y - v0.p.y);
		let right_step_z = (v2.p.z - v1.p.z) / (v2.p.y - v1.p.y);

		let y_start = (v0.p.y - 0.5).ceil() as u32;
		let y_end = (v2.p.y - 0.5).ceil() as u32;

		let mut lhs = v0.clone();
		let lhs_step = (v2 - v0) / (y_end - y_start) as f32;

		let mut rhs = v1.clone();
		let rhs_step = (v2 - v1) / (y_end - y_start) as f32;

		for y in y_start..y_end {

			let delta_y = y as f32 + 0.5 - v0.p.y;

			let x_left = v0.p.x + left_step_x * delta_y;
			let x_right = v1.p.x + right_step_x * delta_y;
			let x_span = x_right - x_left;

			let z_start: f32 =  v0.p.z + left_step_z * delta_y;
			let z_end: f32 = v1.p.z + right_step_z * delta_y;
			let z_span: f32 = z_end - z_start;

			let x_start = (x_left - 0.5).ceil() as u32;
			let x_end = (x_right - 0.5).ceil() as u32;

			let mut cursor = lhs.clone();
			let cursor_step = (rhs - cursor) / (x_end - x_start) as f32;

			for x in x_start..x_end {

				let x_relative = x - x_start;
				let x_progress: f32 = x_relative as f32 / x_span as f32;

				let z = z_start + z_span * x_progress;

				let color = self.get_pixel_color(cursor);

				self.set_pixel(x, y, z, color);

				cursor = cursor + cursor_step;

			}

			lhs = lhs + lhs_step;
			rhs = rhs + rhs_step;

		}

	}

	#[inline(always)]
	fn flat_bottom_triangle_fill(
		&mut self,
		v0: Vertex,
		v1: Vertex,
		v2: Vertex) -> ()
	{

		// @NOTE(mzalla) v0 as top vertex
		// @NOTE(mzalla) v1 as bottom-left vertex
		// @NOTE(mzalla) v2 as bottom-right vertex

		let left_step_x = (v1.p.x - v0.p.x) / (v1.p.y - v0.p.y);
		let right_step_x = (v2.p.x - v0.p.x) / (v2.p.y - v0.p.y);

		let left_step_z = (v1.p.z - v0.p.z) / (v1.p.y - v0.p.y);
		let right_step_z = (v2.p.z - v0.p.z) / (v2.p.y - v0.p.y);

		let y_start = (v0.p.y - 0.5).ceil() as u32;
		let y_end = (v2.p.y - 0.5).ceil() as u32;

		let mut lhs = v0.clone();
		let lhs_step = (v1 - v0) / (y_end - y_start) as f32;

		let mut rhs = v0.clone();
		let rhs_step = (v2 - v0) / (y_end - y_start) as f32;

		for y in y_start..y_end {

			let delta_y = y as f32 + 0.5 - v0.p.y;

			let x_left =  v0.p.x + left_step_x * delta_y;
			let x_right = v0.p.x + right_step_x * delta_y;
			let x_span = x_right - x_left;

			let z_start: f32 =  v0.p.z + left_step_z * delta_y;
			let z_end: f32 = v0.p.z + right_step_z * delta_y;
			let z_span: f32 = z_end - z_start;

			let x_start = (x_left - 0.5).ceil() as u32;
			let x_end = (x_right - 0.5).ceil() as u32;

			let mut cursor = lhs.clone();
			let cursor_step = (rhs - cursor) / (x_end - x_start) as f32;

			for x in x_start..x_end {

				let x_relative = x - x_start;
				let x_progress: f32 = x_relative as f32 / x_span as f32;

				let z = z_start + z_span * x_progress;

				let color = self.get_pixel_color(cursor);

				self.set_pixel(x, y, z, color);

				cursor = cursor + cursor_step;

			}

			lhs = lhs + lhs_step;
			rhs = rhs + rhs_step;

		}

	}

	#[inline(always)]
	fn triangle_fill(
		&mut self,
		v0: Vertex,
		v1: Vertex,
		v2: Vertex) -> ()
	{

		let mut tri = vec![
			v0,
			v1,
			v2,
		];

		// Sorts points by y-value (highest-to-lowest)

		if tri[1].p.y < tri[0].p.y {
			tri.swap(0, 1);
		}
		if tri[2].p.y < tri[1].p.y {
			tri.swap(1, 2);
		}
		if tri[1].p.y < tri[0].p.y {
			tri.swap(0, 1);
		}

		if tri[0].p.y == tri[1].p.y {

			// Flat-top (horizontal line is tri[0]-to-tri[1]);

			// tri[2] must sit below tri[0] and tri[1]; tri[0] and tri[1] cannot
			// have the same x-value; therefore, sort tri[0] and tri[1] by x-value;

			if tri[1].p.x < tri[0].p.x {
				tri.swap(0, 1);
			}

			self.flat_top_triangle_fill(tri[0], tri[1], tri[2]);

		}
		else if tri[1].p.y == tri[2].p.y {

			// Flat-bottom (horizontal line is tri[1]-to-tri[2]);

			// tri[0] must sit above tri[1] and tri[2]; tri[1] and tri[2] cannot
			// have the same x-value; therefore, sort tri[1] and tri[2] by x-value;

			if tri[2].p.x < tri[1].p.x {
				tri.swap(1, 2);
			}

			self.flat_bottom_triangle_fill(tri[0], tri[1], tri[2]);

		}
		else
		{

			// panic!("y0={}, y1={}, y2={}", tri[0].y, tri[1].y, tri[2].y);

			// Find splitting vertex

			let alpha_split =
				(tri[1].p.y - tri[0].p.y) /
				(tri[2].p.y - tri[0].p.y);

			let split_vertex = Vertex::interpolate(tri[0], tri[2], alpha_split);

			if tri[1].p.x < split_vertex.p.x {

				// Major right

				// tri[0] must sit above tri[1] and split_point; tri[1] and
				// split_point cannot have the same x-value; therefore, sort tri[1]
				// and split_point by x-value;

				self.flat_bottom_triangle_fill(tri[0], tri[1], split_vertex);

				self.flat_top_triangle_fill(tri[1], split_vertex, tri[2]);

			}
			else
			{

				// Major left

				self.flat_bottom_triangle_fill(tri[0], split_vertex, tri[1]);

				self.flat_top_triangle_fill(split_vertex, tri[1], tri[2]);

			}

		}

	}

	fn get_pixel_color(
		&self,
		it: Vertex) -> Color
	{

		// Calculate luminance

		let min_luminance = 150.0;
		let max_luminance = 255.0;

		let light_intensity = 1.0;

		let luminance = -1.0 * light_intensity * self.directional_light_normal.dot(it.n);

		let scaled_luminance: f32 = min_luminance + luminance * (max_luminance - min_luminance);

		let color = Color::RGB(
			scaled_luminance as u8,
			scaled_luminance as u8,
			scaled_luminance as u8
			// (0.5 * scaled_luminances) as u8
		);

		return color;

	}

}
