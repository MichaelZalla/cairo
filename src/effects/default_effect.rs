use crate::{
	lib::{
		effect::Effect, color::Color, vec::vec3::Vec3, matrix::Mat3,
	},
	vertices::default_vertex::DefaultVertex
};

pub struct DefaultEffect {
	scale: Vec3,
	rotation: Mat3,
	translation: Vec3,
	mesh_color: Vec3,
	ambient_light: Vec3,
	diffuse_light: Vec3,
	diffuse_light_direction: Vec3,
	point_light: Vec3,
	point_light_position: Vec3,
	constant_attenuation: f32,
	linear_attenuation: f32,
	quadratic_attenuation: f32,
	specular_intensity: f32,
	specular_power: i32,
}

impl DefaultEffect {

	pub fn new(
		scale: Vec3,
		rotation: Mat3,
		translation: Vec3,
		mesh_color: Vec3,
		ambient_light: Vec3,
		diffuse_light: Vec3,
		diffuse_light_direction: Vec3,
		point_light: Vec3,
		point_light_position: Vec3,) -> Self
	{
		return DefaultEffect {
			scale,
			rotation,
			translation,
			mesh_color,
			ambient_light,
			diffuse_light,
			diffuse_light_direction,
			point_light,
			point_light_position,
			constant_attenuation: 0.382,
			linear_attenuation: 1.0,
			quadratic_attenuation: 2.619,
			specular_intensity: 1.0,
			specular_power: 10,
		};
	}

	pub fn set_scale(
		&mut self,
		matrix: Vec3) -> ()
	{
		self.scale = matrix;
	}

	fn get_rotation(
		&self) -> Mat3
	{
		return self.rotation;
	}

	pub fn set_rotation(
		&mut self,
		mat: Mat3) -> ()
	{
		self.rotation = mat;
	}

	pub fn set_translation(
		&mut self,
		vec: Vec3) -> ()
	{
		self.translation = vec;
	}

	pub fn set_mesh_color(
		&mut self,
		color: Vec3) -> ()
	{
		self.mesh_color = color;
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

	pub fn set_point_light(
		&mut self,
		light: Vec3) -> ()
	{
		self.point_light = light;
	}

	pub fn set_point_light_position(
		&mut self,
		pos: Vec3) -> ()
	{
		self.point_light_position = pos;
	}

}

impl Effect for DefaultEffect {

	type Vertex = DefaultVertex;

	fn vs(&self, v: Self::Vertex) -> Self::Vertex {

		let mut vertex = Self::Vertex::new();

		vertex.p = v.p.clone();

		vertex.p *= self.rotation;

		vertex.p *= self.scale;
		vertex.p += self.translation;

		vertex.world_pos = vertex.p.clone();

		vertex.n = v.n.clone();

		vertex.n *= self.rotation;

		vertex.n = vertex.n.as_normal();

		vertex.c = v.c.clone();

		return vertex;

	}

	fn ps(&self, interpolant: <Self as Effect>::Vertex) -> Color {

		let surface_normal = interpolant.n.as_normal();

		// Calculate diffuse light intensity

		let diffuse_intensity = self.diffuse_light * (0.0 as f32).max(
			(interpolant.n.as_normal() * -1.0).dot(
				self.diffuse_light_direction
			)
		);

		// Calculate point light intensity

		let vertex_to_point_light = self.point_light_position - interpolant.world_pos;
		let distance_to_point_light = vertex_to_point_light.mag();
		let normal_to_point_light = vertex_to_point_light / distance_to_point_light;

		let attentuation = 1.0 / (
			self.quadratic_attenuation * distance_to_point_light.powi(2) +
			self.linear_attenuation * distance_to_point_light +
			self.constant_attenuation
		);

		let point_intensity = self.point_light * attentuation * (0.0 as f32).max(
			surface_normal.dot(normal_to_point_light)
		);

		// Calculate specular light intensity

		// point light projected onto surface normal
		let w = surface_normal * self.point_light.dot(surface_normal);

		// vector to reflected light ray
		let r = w * 2.0 - vertex_to_point_light;

		// normal for reflected light
		let r_inverse_hat = r.as_normal() * -1.0;

		let specular_intensity =
			self.point_light *
			self.specular_intensity *
			(0.0 as f32).max(
				r_inverse_hat.dot(interpolant.world_pos.as_normal())
			).powi(self.specular_power);

		// Calculate our color based on mesh color and light intensities

		let color = (*self.mesh_color.get_hadamard(
			self.ambient_light + diffuse_intensity + point_intensity + specular_intensity
		).saturate()) * 255.0;

		return Color{
			r: color.x as u8,
			g: color.y as u8,
			b: color.z as u8,
			a: 255 as u8,
		};

	}

}
