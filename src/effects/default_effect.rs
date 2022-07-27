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
	point_light: Vec3,
	point_light_position: Vec3,
	constant_attenuation: f32,
	linear_attenuation: f32,
	quadratic_attenuation: f32,
}

impl DefaultEffect {

	pub fn new(
		scale: Vec3,
		rotation: Vec3,
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

		vertex.world_pos = vertex.p.clone();

		vertex.n = v.n.clone();

		vertex.n.rotate_along_z(self.rotation.z);
		vertex.n.rotate_along_x(self.rotation.x);
		vertex.n.rotate_along_y(self.rotation.y);

		vertex.n = vertex.n.as_normal();

		vertex.c = v.c.clone();

		return vertex;

	}

	fn ps(&self, interpolant: <Self as Effect>::Vertex) -> Color {

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
			// interpolant.n.dot(normal_to_point_light)
			interpolant.n.as_normal().dot(normal_to_point_light)
		);

		// Calculate our color based on mesh color and light intensities

		let color = (*self.mesh_color.get_hadamard(
			point_intensity + diffuse_intensity + self.ambient_light
		).saturate()) * 255.0;

		return Color{
			r: color.x as u8,
			g: color.y as u8,
			b: color.z as u8,
			a: 255 as u8,
		};

	}

}
