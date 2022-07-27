use crate::{
	lib::{
		effect::Effect, color::Color, vec::vec3::Vec3,
	},
	vertices::default_vertex::DefaultVertex
};

pub struct DefaultEffect {
	scale: Vec3,
	rotation: Vec3,
	translation: Vec3,
	mesh_color: Vec3,
	ambient_light: Vec3,
	diffuse_light: Vec3,
	diffuse_light_direction: Vec3,
}

impl DefaultEffect {

	pub fn new(
		scale: Vec3,
		rotation: Vec3,
		translation: Vec3,
		mesh_color: Vec3,
		ambient_light: Vec3,
		diffuse_light: Vec3,
		diffuse_light_direction: Vec3) -> Self
	{
		return DefaultEffect {
			scale,
			rotation,
			translation,
			mesh_color,
			ambient_light,
			diffuse_light,
			diffuse_light_direction,
		};
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

	pub fn set_mesh_color(
		&mut self,
		c: Vec3) -> ()
	{
		self.mesh_color = c;
	}

	pub fn set_ambient_light(
		&mut self,
		light: Vec3) -> ()
	{
		self.ambient_light = light;
	}

	pub fn set_diffuse_light(
		&mut self,
		light: Vec3) -> ()
	{
		self.diffuse_light = light;
	}

	pub fn set_diffuse_light_direction(
		&mut self,
		normal: Vec3) -> ()
	{
		self.diffuse_light_direction = normal;
	}

}

impl Effect for DefaultEffect {

	type Vertex = DefaultVertex;

	fn get_rotation(&self) -> Vec3
	{
		return self.rotation;
	}

	fn vs(&self, v: Self::Vertex) -> Self::Vertex {

		let mut vertex = Self::Vertex::new();

		vertex.p = v.p.clone();

		vertex.p.rotate_along_z(self.rotation.z);
		vertex.p.rotate_along_x(self.rotation.x);
		vertex.p.rotate_along_y(self.rotation.y);

		vertex.p *= self.scale;
		vertex.p += self.translation;

		vertex.n = v.n.clone();

		vertex.n.rotate_along_z(self.rotation.z);
		vertex.n.rotate_along_x(self.rotation.x);
		vertex.n.rotate_along_y(self.rotation.y);

		vertex.n = vertex.n.as_normal();

		let diffuse_intensity = self.diffuse_light * (0.0 as f32).max(
			(vertex.n * -1.0).dot(
				self.diffuse_light_direction
			)
		);

		let color = *(diffuse_intensity + self.ambient_light).saturate() * 255.0;

		vertex.c = color;

		return vertex;

	}

	fn ps(&self, interpolant: <Self as Effect>::Vertex) -> Color {

		return Color{
			r: interpolant.c.x as u8,
			g: interpolant.c.y as u8,
			b: interpolant.c.z as u8,
			a: 255 as u8,
		};

	}

}
