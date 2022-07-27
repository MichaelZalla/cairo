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
}

impl DefaultEffect {

	pub fn new(
		scale: Vec3,
		rotation: Vec3,
		translation: Vec3,
		mesh_color: Vec3,
		ambient_light: Vec3) -> Self
	{
		return DefaultEffect {
			scale,
			rotation,
			translation,
			mesh_color,
			ambient_light,
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

		vertex.c = self.mesh_color.clone();

		return vertex;

	}

	fn ps(&self, interpolant: <Self as Effect>::Vertex) -> Color {

		// Calculate luminance

		let diffuse = 25.0;

		let light_intensity = 0.667;

		let luminance = -1.0 * light_intensity * self.ambient_light.dot(interpolant.n);

		let scaled_luminance: f32 = diffuse + luminance * (255.0 - diffuse);

		let color = Color::RGB(
			scaled_luminance as u8,
			scaled_luminance as u8,
			scaled_luminance as u8
		);

		return color;

	}

}
