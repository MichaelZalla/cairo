use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{
	lib::{
		scene::Scene,
		vec::{
			vec3::Vec3,
			vec4::Vec4,
			vec2::Vec2
		},
		device::{
			KeyboardState,
			MouseState,
			GameControllerState
		},
		graphics::Graphics,
		pipeline::{Pipeline, PipelineOptions},
		matrix::Mat4,
		entity::Entity,
	},
	effects::default_effect::DefaultEffect,
};

static FIELD_OF_VIEW: f32 = 100.0;
static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 10.0;

pub struct DefaultScene {

	pipeline: Pipeline<DefaultEffect>,
	pipeline_options: PipelineOptions,

	screen_width: u32,
	screen_height: u32,
	horizontal_fov_rad: f32,
	vertical_fov_rad: f32,

	entity: Entity,

	camera_position: Vec4,
	camera_rotation_inverse_transform: Mat4,
	camera_movement_speed: f32,
	camera_roll: f32,
	camera_roll_speed: f32,

	point_light_distance_from_camera: f32,

	prev_mouse_state: MouseState,

}

impl DefaultScene {

	pub fn new(
		graphics: Graphics,
		entity: Entity) -> Self
	{

		let camera_position = Vec4::new(Vec3{
			x: 0.0,
			y: 0.0,
			z: 0.0,
		}, 1.0);

		let camera_rotation_inverse_transform = Mat4::identity();

		let camera_movement_speed = 150.0;

		let camera_roll = 0.0;
		let camera_roll_speed = 6.0;

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
			x: 0.25,
			y: -1.0,
			z: -0.25,
		};

		let point_light = Vec3{
			x: 0.4,
			y: 0.4,
			z: 0.4,
		};

		let point_light_position = Vec3::new();

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
			Mat4::translation(entity.position);

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
				ambient_light,
				diffuse_light,
				diffuse_light_direction,
				point_light,
				point_light_position
			),
			pipeline_options
		);

		return DefaultScene{
			pipeline,
			pipeline_options,
			entity,
			camera_position,
			camera_rotation_inverse_transform,
			camera_movement_speed,
			camera_roll,
			camera_roll_speed,
			point_light_distance_from_camera: 20.0,
			screen_width,
			screen_height,
			horizontal_fov_rad,
			vertical_fov_rad,
			prev_mouse_state: MouseState::new(),
		};

	}

}

impl Scene for MeshScene {

	fn update(
		&mut self,
		keyboard_state: &KeyboardState,
		mouse_state: &MouseState,
		game_controller_state: &GameControllerState,
		delta_t_seconds: f32) -> ()
	{

		// Calculate mouse position delta

		let mouse_position = mouse_state.position;

		let nds_mouse_x = mouse_position.0 as f32 / self.screen_width as f32;
		let nds_mouse_y = mouse_position.1 as f32 / self.screen_height as f32;

		let prev_nds_mouse_x = self.prev_mouse_state.position.0 as f32 / self.screen_width as f32;
		let prev_nds_mouse_y = self.prev_mouse_state.position.1 as f32 / self.screen_height as f32;

		let mouse_x_delta = nds_mouse_x - prev_nds_mouse_x;
		let mouse_y_delta = nds_mouse_y - prev_nds_mouse_y;

		// Apply camera rotation based on mouse position delta

		self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
			Mat4::rotation_y(-mouse_x_delta * 2.0 * PI) *
			Mat4::rotation_x(-mouse_y_delta * 2.0 * PI);

		let camera_movement_step = self.camera_movement_speed * delta_t_seconds;
		let camera_roll_step = self.camera_roll_speed * delta_t_seconds;

		let camera_rotation_inverse_transposed =
			self.camera_rotation_inverse_transform.transposed();

		// Translate point light relative to camera based on mousewheel delta

		if mouse_state.wheel_did_move {
			match mouse_state.wheel_direction {
				sdl2::mouse::MouseWheelDirection::Normal => {

					self.point_light_distance_from_camera += mouse_state.wheel_y as f32 / 4.0;

					self.point_light_distance_from_camera = self.point_light_distance_from_camera
						.min(30.0)
						.max(5.0);

				},
				_ => {}
			}
		}

		// Apply camera movement based on keyboard or gamepad input

		let up = Vec4::new(Vec3{ x: 0.0, y: -1.0, z: 0.0 }, 1.0);
		let left = Vec4::new(Vec3{ x: -1.0, y: 0.0, z: 0.0 }, 1.0);
		let forward = Vec4::new(Vec3{ x: 0.0, y: 0.0, z: 1.0 }, 1.0);

		for keycode in &keyboard_state.keys_pressed {
			match keycode {
				Keycode::Up|Keycode::W { .. } => {
					self.camera_position += forward * camera_movement_step * camera_rotation_inverse_transposed;
				},
				Keycode::Down|Keycode::S { .. } => {
					self.camera_position -= forward * camera_movement_step * camera_rotation_inverse_transposed;
				},
				Keycode::Left|Keycode::A { .. } => {
					self.camera_position += left * camera_movement_step * camera_rotation_inverse_transposed;
				},
				Keycode::Right|Keycode::D { .. } => {
					self.camera_position -= left * camera_movement_step * camera_rotation_inverse_transposed;
				},
				Keycode::Q { .. } => {
					self.camera_position -= up * camera_movement_step * camera_rotation_inverse_transposed;
				},
				Keycode::E { .. } => {
					self.camera_position += up * camera_movement_step * camera_rotation_inverse_transposed;
				},
				Keycode::Z { .. } => {
					self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
						Mat4::rotation_z(-camera_roll_step);
				},
				Keycode::C { .. } => {
					self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
						Mat4::rotation_z(camera_roll_step);
				},
				Keycode::Num1 { .. } => {
					self.pipeline_options.should_render_wireframe =
						!self.pipeline_options.should_render_wireframe;

					self.pipeline.set_options(self.pipeline_options);
				},
				Keycode::Num2 { .. } => {
					self.pipeline_options.should_render_shader =
						!self.pipeline_options.should_render_shader;

					self.pipeline.set_options(self.pipeline_options);
				},
				Keycode::Num3 { .. } => {
					self.pipeline_options.should_render_normals =
						!self.pipeline_options.should_render_normals;

					self.pipeline.set_options(self.pipeline_options);
				},
				_ => {}
			}
		}

		if game_controller_state.buttons.x {

			self.pipeline_options.should_render_wireframe =
				!self.pipeline_options.should_render_wireframe;

			self.pipeline.set_options(self.pipeline_options);

		} else if game_controller_state.buttons.y {

			self.pipeline_options.should_render_normals =
				!self.pipeline_options.should_render_normals;

			self.pipeline.set_options(self.pipeline_options);

		}

		if game_controller_state.buttons.dpad_up {
			self.camera_position += forward * camera_movement_step * camera_rotation_inverse_transposed;
		} else if game_controller_state.buttons.dpad_down {
			self.camera_position -= forward * camera_movement_step * camera_rotation_inverse_transposed;
		} else if game_controller_state.buttons.dpad_left {
			self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
				Mat4::rotation_z(-camera_roll_step);
		} else if game_controller_state.buttons.dpad_right {
			self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
				Mat4::rotation_z(camera_roll_step);
		}

		let left_joystick_position_normalized = Vec2{
			x: game_controller_state.joysticks.left.position.x as f32 / std::i16::MAX as f32,
			y: game_controller_state.joysticks.left.position.y as f32 / std::i16::MAX as f32,
			z: 1.0,
		};

		if left_joystick_position_normalized.x > 0.5 {
			self.camera_position -= left * camera_movement_step * camera_rotation_inverse_transposed;
		} else if left_joystick_position_normalized.x < -0.5 {
			self.camera_position += left * camera_movement_step * camera_rotation_inverse_transposed;
		}

		if left_joystick_position_normalized.y > 0.5 {
			self.camera_position -= forward * camera_movement_step * camera_rotation_inverse_transposed;
		} else if left_joystick_position_normalized.y < -0.5 {
			self.camera_position += forward * camera_movement_step * camera_rotation_inverse_transposed;
		}

		let right_joystick_position_normalized = Vec2{
			x: game_controller_state.joysticks.right.position.x as f32 / std::i16::MAX as f32,
			y: game_controller_state.joysticks.right.position.y as f32 / std::i16::MAX as f32,
			z: 1.0,
		};

		let yaw_delta = -right_joystick_position_normalized.x * PI / 32.0;
		let pitch_delta = -right_joystick_position_normalized.y * PI / 32.0;
		let roll_delta = -yaw_delta * 0.5;

		self.camera_roll += roll_delta;
		self.camera_roll = self.camera_roll % (2.0 * PI);

		self.camera_rotation_inverse_transform = self.camera_rotation_inverse_transform *
			Mat4::rotation_y(yaw_delta) *
			Mat4::rotation_x(pitch_delta) *
			Mat4::rotation_z(-yaw_delta * 0.5);

		// Mesh rotation via time delta

		self.entity.rotation.z += 0.2 * PI * delta_t_seconds;
		self.entity.rotation.z %= 2.0 * PI;

		self.entity.rotation.x += 0.2 * PI * delta_t_seconds;
		self.entity.rotation.x %= 2.0 * PI;

		self.entity.rotation.y += 0.2 * PI * delta_t_seconds;
		self.entity.rotation.y %= 2.0 * PI;

		let world_transform =
			Mat4::scaling(0.5) *
			Mat4::rotation_x(self.entity.rotation.x) *
			Mat4::rotation_y(self.entity.rotation.y) *
			Mat4::rotation_z(self.entity.rotation.z) *
			Mat4::translation(self.entity.position);

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

		self.pipeline.clear_pixel_buffer();

		if self.pipeline_options.should_render_shader {
			self.pipeline.clear_z_buffer();
		}

		self.pipeline.render(&self.entity.mesh);

	}

	fn get_pixel_data(&self) -> &Vec<u32> {

		return self.pipeline.get_pixel_data();

	}

}
