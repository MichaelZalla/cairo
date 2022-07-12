use super::{graphics::Graphics, vec::{vec3::Vec3, vec2::Vec2}, mesh::Mesh, color::{self, Color}};

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

#[derive(Copy, Clone, Default)]
pub struct PipelineOptions {
	pub should_render_wireframe: bool,
	pub should_render_shader: bool,
	pub should_render_normals: bool,
}

pub struct Pipeline {

	options: PipelineOptions,

	graphics: Graphics,

	buffer_width_over_2: f32,
	buffer_height_over_2: f32,

	scale: Vec3,
	rotation: Vec3,
	translation: Vec3,

	light_normal: Vec3,

	z_buffer: Vec<f32>,
	z_buffer_size: usize,

}

impl Pipeline {

	pub fn new(graphics: Graphics, options: PipelineOptions) -> Self {

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
			light_normal: Vec3{
				x: 0.0,
				y: 0.0,
				z: 1.0
			},
			z_buffer: z_buffer,
			z_buffer_size: z_buffer_size,
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
		self.light_normal = normal;
	}

	pub fn render(
		&mut self,
		mesh: &Mesh) -> ()
	{

		self.graphics.buffer.clear();

		if self.options.should_render_shader {
			for i in 0..self.z_buffer_size {
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
				point.x / point.z * self.graphics.buffer.height_over_width + 1.0
			) * self.buffer_width_over_2;

			point.y = (
				(-1.0 * point.y) / point.z + 1.0
			) * self.buffer_height_over_2;

		}

		if self.options.should_render_wireframe {
			self.graphics.poly_line(points.as_slice(), color::WHITE);
		}

		// Interpolate entire Vertex (all attributes) when drawing (scanline
		// interpolant)

		if self.options.should_render_shader {

			// Calculate luminance

			let min_luminance = 150.0;
			let max_luminance = 255.0;

			let light_intensity = 1.0;

			let luminance0 = -1.0 * light_intensity * self.light_normal.dot(triangle[0].n);
			let luminance1 = -1.0 * light_intensity * self.light_normal.dot(triangle[1].n);
			let luminance2 = -1.0 * light_intensity * self.light_normal.dot(triangle[2].n);

			let luminance_avg = (luminance0 + luminance1 + luminance2) / 3.0;

			let scaled_luminance: f32 = min_luminance + luminance_avg * (max_luminance - min_luminance);

			// println!("luminance_avg = {}", luminance_avg);

			let color = Color::RGB(
				scaled_luminance as u8,
				scaled_luminance as u8,
				scaled_luminance as u8
				// (0.5 * scaled_luminances) as u8
			);

			let z_buffer_width = self.graphics.buffer.width;

			self.graphics.triangle_fill(
				self.z_buffer.as_mut_slice(),
				z_buffer_width,
				points.as_slice(),
				color
			);

		}

		if self.options.should_render_normals {

			for i in 0..=2 {

				let world_vertex_relative_normal = triangle[i].p + (triangle[i].n * 0.05);

				let screen_vertex_relative_normal = Vec2{
					x: (
						world_vertex_relative_normal.x / world_vertex_relative_normal.z * self.graphics.buffer.height_over_width + 1.0
					) * self.buffer_width_over_2,
					y: (
						(-1.0 * world_vertex_relative_normal.y) / world_vertex_relative_normal.z + 1.0
					) * self.buffer_height_over_2,
					z: 0.0,
				};

				let from_point = points[i];

				let to_point = screen_vertex_relative_normal;

				self.graphics.line(
					from_point.x as u32,
				from_point.y as u32,
				to_point.x as u32,
				to_point.y as u32,
				color::RED);

			}

		}

	}

}
