use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{
	lib::{
		scene::Scene,
		vec::{vec3::Vec3, vec4::Vec4},
		mesh::Mesh,
		device::{KeyboardState, MouseState},
		graphics::Graphics,
		pipeline::{Pipeline, PipelineOptions},
		matrix::Mat4,
	},
	effects::default_effect::DefaultEffect,
};

static FIELD_OF_VIEW: f32 = 100.0;
static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 10.0;

pub struct MeshScene {

	pipeline: Pipeline<DefaultEffect>,
	pipeline_options: PipelineOptions,

	screen_width: u32,
	screen_height: u32,
	horizontal_fov_rad: f32,
	vertical_fov_rad: f32,

	mesh: Mesh,
	mesh_position: Vec3,
	mesh_rotation: Vec3,

	camera_position: Vec4,
	camera_rotation_inverse_transform: Mat4,
	camera_speed: f32,

	point_light_distance_from_camera: f32,

	prev_mouse_state: MouseState,

}

impl MeshScene {

	pub fn new(
		graphics: Graphics,
		mesh: Mesh) -> Self
	{

		let mesh_position = Vec3{
			x: 0.0,
			y: 0.0,
			z: 10.0,
		};

		let mesh_rotation = Vec3::new();

		let mesh_color = Vec3{
			x: 0.5,
			y: 0.0,
			z: 0.65,
		};

		let camera_position = Vec4::new(Vec3::new(), 1.0);
		let camera_rotation_inverse_transform = Mat4::identity();
		let camera_speed = 15.0;

		let ambient_light = Vec3{
			x: 0.1,
			y: 0.1,
			z: 0.1,
		};

		let diffuse_light = Vec3{
			x: 0.3,
			y: 0.3,
			z: 0.3,
		};

		let diffuse_light_direction = Vec3{
			x: 0.0,
			y: 0.0,
			z: 1.0,
		};

		let point_light = Vec3{
			x: 0.8,
			y: 0.8,
			z: 0.8,
		};

		let point_light_position = Vec3{
			x: 0.0,
			y: 0.0,
			z: -1.0,
		};

		let pipeline_options = crate::lib::pipeline::PipelineOptions {
			should_render_wireframe: true,
			should_render_shader: true,
			should_render_normals: false,
		};

		let buffer = &graphics.buffer;

		let screen_width = buffer.width;
		let screen_height = buffer.height;

		let world_transform =
			Mat4::scaling(0.5) *
			Mat4::translation(mesh_position);

		let camera_position_inverse = camera_position * -1.0;

		let view_transform =
			Mat4::translation(Vec3 {
				x: camera_position_inverse.x,
				y: camera_position_inverse.y,
				z: camera_position_inverse.z,
			}
		);

		let world_view_transform =
			world_transform *
			view_transform;

		// let projection_transform = Mat4::projection(
		// 	2.0 * graphics.buffer.width_over_height,
		// 	2.0,
		// 	1.0,
		// 	10.0,
		// );

		let aspect_ratio = graphics.buffer.width_over_height;

		let horizontal_fov_rad = PI * (FIELD_OF_VIEW) / 180.0;
		let vertical_fov_rad = PI * (FIELD_OF_VIEW / aspect_ratio) / 180.0;

		let projection_transform = Mat4::projection_for_fov(
			FIELD_OF_VIEW,
			aspect_ratio,
			PROJECTION_Z_NEAR,
			PROJECTION_Z_FAR,
		);

		let pipeline = Pipeline::new(
			graphics,
			DefaultEffect::new(
				world_view_transform,
				projection_transform,
				mesh_color,
				ambient_light,
				diffuse_light,
				diffuse_light_direction,
				point_light,
				point_light_position
			),
			pipeline_options
		);

		return MeshScene{
			pipeline,
			pipeline_options,
			mesh,
			mesh_position,
			mesh_rotation,
			camera_position,
			camera_rotation_inverse_transform,
			camera_speed,
			point_light_distance_from_camera: 5.0,
			screen_width,
			screen_height,
			horizontal_fov_rad,
			vertical_fov_rad,
			prev_mouse_state: MouseState::new(),
		};

	}

}

impl Scene for MeshScene {

	fn update(&mut self, keyboard_state: &KeyboardState, mouse_state: &MouseState, delta_t_seconds: f32) -> () {

		let mouse_position = mouse_state.position;

		let nds_mouse_x = mouse_position.0 as f32 / self.screen_width as f32;
		let nds_mouse_y = mouse_position.1 as f32 / self.screen_height as f32;

		let prev_nds_mouse_x = self.prev_mouse_state.position.0 as f32 / self.screen_width as f32;
		let prev_nds_mouse_y = self.prev_mouse_state.position.1 as f32 / self.screen_height as f32;

		let mouse_x_delta = nds_mouse_x - prev_nds_mouse_x;
		let mouse_y_delta = nds_mouse_y - prev_nds_mouse_y;

		self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
			Mat4::rotation_y(-mouse_x_delta  * 2.0 * PI) *
			Mat4::rotation_x(-mouse_y_delta  * 2.0 * PI);

		let camera_step = self.camera_speed * delta_t_seconds;

		let camera_rotation_inverse_transposed =
			self.camera_rotation_inverse_transform.transposed();

		let up = Vec4::new(Vec3{ x: 0.0, y: -1.0, z: 0.0 }, 1.0);
		let left = Vec4::new(Vec3{ x: -1.0, y: 0.0, z: 0.0 }, 1.0);
		let forward = Vec4::new(Vec3{ x: 0.0, y: 0.0, z: 1.0 }, 1.0);

		for keycode in &keyboard_state.keys_pressed {
			match keycode {
				Keycode::Down|Keycode::S { .. } => {
					self.camera_position -= forward * camera_step * camera_rotation_inverse_transposed;
				},
				Keycode::Up|Keycode::W { .. } => {
					self.camera_position += forward * camera_step * camera_rotation_inverse_transposed;
				},
				Keycode::Right|Keycode::D { .. } => {
					self.camera_position -= left * camera_step * camera_rotation_inverse_transposed;
				},
				Keycode::Left|Keycode::A { .. } => {
					self.camera_position += left * camera_step * camera_rotation_inverse_transposed;
				},
				Keycode::Q { .. } => {
					self.camera_position -= up * camera_step * camera_rotation_inverse_transposed;
				},
				Keycode::E { .. } => {
					self.camera_position += up * camera_step * camera_rotation_inverse_transposed;
				},
				Keycode::Z { .. } => {
					self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
						Mat4::rotation_z(camera_step / 2.0);
				},
				Keycode::C { .. } => {
					self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
						Mat4::rotation_z(-camera_step / 2.0);
				},
				Keycode::Num1 { .. } => {
					self.pipeline_options.should_render_wireframe =
						!self.pipeline_options.should_render_wireframe;

					self.pipeline.set_options(self.pipeline_options);
				}
				Keycode::Num2 { .. } => {
					self.pipeline_options.should_render_shader =
						!self.pipeline_options.should_render_shader;

					self.pipeline.set_options(self.pipeline_options);
				}
				Keycode::Num3 { .. } => {
					self.pipeline_options.should_render_normals =
						!self.pipeline_options.should_render_normals;

					self.pipeline.set_options(self.pipeline_options);
				}
				_ => {}
			}
		}

		if mouse_state.wheel_did_move {
			match mouse_state.wheel_direction {
				sdl2::mouse::MouseWheelDirection::Normal => {
					self.camera_position.z -= (mouse_state.wheel_y as f32) / 4.0;
				},
				_ => {}
			}
		}

		// Mesh rotation via time delta

		self.mesh_rotation.z += 0.2 * PI * delta_t_seconds;
		self.mesh_rotation.z %= 2.0 * PI;

		self.mesh_rotation.x += 0.2 * PI * delta_t_seconds;
		self.mesh_rotation.x %= 2.0 * PI;

		self.mesh_rotation.y += 0.2 * PI * delta_t_seconds;
		self.mesh_rotation.y %= 2.0 * PI;

		let world_transform =
			Mat4::scaling(0.5) *
			Mat4::rotation_x(self.mesh_rotation.x) *
			Mat4::rotation_y(self.mesh_rotation.y) *
			Mat4::rotation_z(self.mesh_rotation.z) *
			Mat4::translation(self.mesh_position);

		let camera_translation_inverse = self.camera_position * -1.0;

		let camera_translation_inverse_transform =
			Mat4::translation(Vec3{
				x: camera_translation_inverse.x,
				y: camera_translation_inverse.y,
				z: camera_translation_inverse.z,
			});

		let view_transform =
			camera_translation_inverse_transform *
			self.camera_rotation_inverse_transform;

		let world_view_transform =
			world_transform *
			view_transform;

		self.pipeline.effect.set_world_view_transform(
			world_view_transform
		);

		// // Diffuse light direction rotation via mouse input

		// let mut rotated_diffuse_light_direction = Vec3{
		// 	x: 0.0,
		// 	y: 0.0,
		// 	z: 1.0,
		// };

		// rotated_diffuse_light_direction.rotate_along_x(-2.0 * PI * nds_mouse_y * -1.0);
		// rotated_diffuse_light_direction.rotate_along_y(-2.0 * PI * nds_mouse_x);

		// self.pipeline.effect.set_diffuse_light_direction(
		// 	rotated_diffuse_light_direction
		// );

		// Point light position translation via mouse input

		let point_light_position = forward * self.point_light_distance_from_camera;

		// point_light_position.y = 5.0 - (10.0 * nds_mouse_y);
		// point_light_position.x = -5.0 + (10.0 * nds_mouse_x);

		self.pipeline.effect.set_point_light_position(
			Vec3{
				x: point_light_position.x,
				y: point_light_position.x,
				z: point_light_position.z,
			}
		);

		self.prev_mouse_state = mouse_state.clone();

	}

	fn render(&mut self) -> () {

		self.pipeline.render(&self.mesh);

	}

	fn get_pixel_data(&self) -> &Vec<u32> {

		return self.pipeline.get_pixel_data();

	}

}
