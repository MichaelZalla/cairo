use crate::{
	lib::{
		effect::Effect,
		color::Color,
		vec::{vec3::Vec3, vec4::Vec4},
		matrix::Mat4,
	},
	vertices::{
		default_vertex_in::DefaultVertexIn,
		default_vertex_out::DefaultVertexOut
	}
};

pub struct DefaultEffect {
	transform: Mat4,
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
		transform: Mat4,
		mesh_color: Vec3,
		ambient_light: Vec3,
		diffuse_light: Vec3,
		diffuse_light_direction: Vec3,
		point_light: Vec3,
		point_light_position: Vec3,) -> Self
	{
		return DefaultEffect {
			transform,
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

	pub fn set_transform(
		&mut self,
		mat: Mat4) -> ()
	{
		self.transform = mat;
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

	type VertexIn = DefaultVertexIn;
	type VertexOut = DefaultVertexOut;

	fn vs(&self, v: Self::VertexIn) -> Self::VertexOut {

		let mut out = Self::VertexOut::new();

		out.p = Vec4::new(v.p, 1.0) * self.transform;

		out.world_pos = Vec3{
			x: out.p.x,
			y: out.p.y,
			z: out.p.z,
		};

		out.n = Vec4::new(v.n, 0.0) * self.transform;

		out.n = out.n.as_normal();

		out.c = v.c.clone();

		return out;

	}

	fn ps(&self, interpolant: <Self as Effect>::VertexOut) -> Color {

		let surface_normal = interpolant.n.as_normal();

		let surface_normal_vec3 = Vec3{
			x: surface_normal.x,
			y: surface_normal.y,
			z: surface_normal.z,
		};

		// Calculate diffuse light intensity

		let diffuse_intensity = self.diffuse_light * (0.0 as f32).max(
			(surface_normal_vec3 * -1.0).dot(
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
			surface_normal_vec3.dot(normal_to_point_light)
		);

		// Calculate specular light intensity

		// point light projected onto surface normal
		let w = surface_normal_vec3 * self.point_light.dot(surface_normal_vec3);

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
