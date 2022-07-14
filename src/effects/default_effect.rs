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
	ambient_light: Vec3,
}

impl DefaultEffect {

	pub fn new(
		scale: Vec3,
		rotation: Vec3,
		translation: Vec3,
		ambient_light: Vec3) -> Self
	{
		return DefaultEffect {
			scale,
			rotation,
			translation,
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

	fn vs(&self, v: Vec3) -> Self::Vertex {

		let mut vertex = Self::Vertex::new();

		vertex.p = v.clone();

		vertex.p.rotate_along_z(self.rotation.z);
		vertex.p.rotate_along_x(self.rotation.x);
		vertex.p.rotate_along_y(self.rotation.y);

		vertex.p *= self.scale;

		vertex.p += self.translation;

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
