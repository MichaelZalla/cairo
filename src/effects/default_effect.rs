use crate::{
	lib::{
		effect::Effect,
		color::{Color, self},
		vec::{vec3::Vec3, vec4::Vec4},
		matrix::Mat4,
	},
	vertices::{
		default_vertex_in::DefaultVertexIn,
		default_vertex_out::DefaultVertexOut
	}
};

pub struct DefaultEffect {
	world_view_transform: Mat4,
	projection_transform: Mat4,
	world_view_projection_transform: Mat4,
	ambient_light: Vec3,
	diffuse_light: Vec3,
	diffuse_light_direction: Vec4,
	point_light: Vec3,
	point_light_position: Vec3,
	constant_attenuation: f32,
	linear_attenuation: f32,
	quadratic_attenuation: f32,
	specular_intensity: f32,
	specular_power: i32,
	fog_near_z: f32,
	fog_far_z: f32,
	fog_color_vec: Vec3,
}

impl DefaultEffect {

	pub fn new(
		world_view_transform: Mat4,
		projection_transform: Mat4,
		ambient_light: Vec3,
		diffuse_light: Vec3,
		diffuse_light_direction: Vec4,
		point_light: Vec3,
		point_light_position: Vec3) -> Self
	{

		return DefaultEffect {
			world_view_transform,
			projection_transform,
			world_view_projection_transform: world_view_transform * projection_transform,
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
			fog_near_z: 25.0,
			fog_far_z: 150.0,
			fog_color_vec: color::SKY_BOX.to_vec3(),
		};

	}

	pub fn set_world_view_transform(
		&mut self,
		mat: Mat4)
	{
		self.world_view_transform = mat;

		self.world_view_projection_transform = self.world_view_transform * self.projection_transform;
	}

	pub fn set_projection_transform(
		&mut self,
		mat: Mat4)
	{
		self.projection_transform = mat;

		self.world_view_projection_transform = self.world_view_transform * self.projection_transform;
	}

	pub fn set_ambient_light(
		&mut self,
		light: Vec3)
	{
		self.ambient_light = light;
	}

	pub fn set_diffuse_light(
		&mut self,
		light: Vec3)
	{
		self.diffuse_light = light;
	}

	pub fn set_diffuse_light_direction(
		&mut self,
		normal: Vec4)
	{
		self.diffuse_light_direction = normal;
	}

	pub fn set_point_light(
		&mut self,
		light: Vec3)
	{
		self.point_light = light;
	}

	pub fn set_point_light_position(
		&mut self,
		pos: Vec3)
	{
		self.point_light_position = pos;
	}

}

impl Effect for DefaultEffect {

	type VertexIn = DefaultVertexIn;
	type VertexOut = DefaultVertexOut;

	fn get_projection(&self) -> Mat4 {
		return self.projection_transform;
	}

	fn vs(&self, v: Self::VertexIn) -> Self::VertexOut {

		let mut out = Self::VertexOut::new();

		out.p = Vec4::new(v.p, 1.0) * self.world_view_projection_transform;

		let world_pos = Vec4::new(v.p, 1.0) * self.world_view_transform;

		out.world_pos = Vec3{
			x: world_pos.x,
			y: world_pos.y,
			z: world_pos.z,
		};

		out.n = Vec4::new(v.n, 0.0) * self.world_view_transform;

		out.n = out.n.as_normal();

		out.c = v.c.clone();

		return out;

	}

	fn ps(&self, interpolant: &<Self as Effect>::VertexOut) -> Color {

		let out = interpolant;

		let surface_normal = out.n;

		let surface_normal_vec3 = Vec3{
			x: surface_normal.x,
			y: surface_normal.y,
			z: surface_normal.z,
		};

		// Calculate diffuse light intensity

		let diffuse_light_direction_world_view =
			(
				self.diffuse_light_direction *
				self.world_view_transform
			).as_normal();

		let diffuse_intensity = self.diffuse_light * (0.0 as f32).max(
			(surface_normal_vec3 * -1.0).dot(
				Vec3 {
					x: diffuse_light_direction_world_view.x,
					y: diffuse_light_direction_world_view.y,
					z: diffuse_light_direction_world_view.z,
				}
			)
		);

		// Calculate point light intensity

		let vertex_to_point_light = self.point_light_position - out.world_pos;
		let distance_to_point_light = vertex_to_point_light.mag();
		let normal_to_point_light = vertex_to_point_light / distance_to_point_light;

		let likeness = normal_to_point_light.dot(surface_normal_vec3 * -1.0);

		let attentuation = 1.0 / (
			self.quadratic_attenuation * distance_to_point_light.powi(2) +
			self.linear_attenuation * distance_to_point_light +
			self.constant_attenuation
		);

		let mut point_intensity: Vec3 = Vec3::new();
		let mut specular_intensity: Vec3 = Vec3::new();

		if likeness < 0.0 {

			point_intensity = self.point_light * attentuation * (0.0 as f32).max(
				surface_normal_vec3.dot(normal_to_point_light)
			);

			// Calculate specular light intensity

			// point light projected onto surface normal
			let w = surface_normal_vec3 * self.point_light.dot(surface_normal_vec3);

			// vector to reflected light ray
			let r = w * 2.0 - vertex_to_point_light;

			// normal for reflected light
			let r_inverse_hat = r.as_normal() * -1.0;

			specular_intensity =
				self.point_light *
				self.specular_intensity *
				(0.0 as f32).max(
					r_inverse_hat.dot(out.world_pos.as_normal())
				).powi(self.specular_power);

		}

		// Calculate our color based on mesh color and light intensities

		out.c = v.c.clone();

		let color = *out.c.get_hadamard(
			self.ambient_light + diffuse_intensity + point_intensity + specular_intensity
		).saturate() * 255.0;

		let distance: f32 = out.world_pos.mag();

		let fog_alpha;

		if distance <= self.fog_near_z {
			fog_alpha = 0.0;
		} else if distance >= self.fog_far_z {
			fog_alpha = 1.0;
		} else {
			fog_alpha = (distance - self.fog_near_z) / (self.fog_far_z - self.fog_near_z);
		}

		let color = Vec3::interpolate(
			out.c,
			self.fog_color_vec,
			fog_alpha
		);

		return Color {
			r: color.x as u8,
			g: color.y as u8,
			b: color.z as u8,
			a: 255 as u8,
		};

	}

}
