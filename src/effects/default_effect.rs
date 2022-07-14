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
	directional_light: Vec3,
}

impl DefaultEffect {

	pub fn new(
		scale: Vec3,
		rotation: Vec3,
		translation: Vec3,
		directional_light: Vec3) -> Self
	{
		return DefaultEffect {
			scale,
			rotation,
			translation,
			directional_light,
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

	pub fn set_directional_light(
		&mut self,
		light: Vec3) -> ()
	{
		self.directional_light = light;
	}

}

impl Effect for DefaultEffect {

	type Vertex = DefaultVertex;

	fn get_rotation(&self) -> Vec3
	{
		return self.rotation;
	}

	fn vs(&self, v: Vec3) -> Self::Vertex {

		let mut result = Self::Vertex::new();

		result.p = v.clone();

		result.p.rotate_along_z(self.rotation.z);
		result.p.rotate_along_x(self.rotation.x);
		result.p.rotate_along_y(self.rotation.y);

		result.p *= self.scale;

		result.p += self.translation;

		return result;

	}

	fn ps(&self, interpolant: <Self as Effect>::Vertex) -> Color {

		// Calculate luminance

		let min_luminance = 150.0;
		let max_luminance = 255.0;

		let light_intensity = 1.0;

		let luminance = -1.0 * light_intensity * self.directional_light.dot(interpolant.n);

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
