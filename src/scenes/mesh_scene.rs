use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{
	lib::{
		scene::Scene,
		vec::vec3::Vec3,
		mesh::{Mesh, get_mesh_from_obj},
		device::{KeyboardState, MouseState},
		graphics::Graphics,
		pipeline::{Pipeline, PipelineOptions},
		matrix::Mat3,
	},
	effects::default_effect::DefaultEffect,
};

pub struct MeshScene {
	pipeline: Pipeline<DefaultEffect>,
	pipeline_options: PipelineOptions,
	mesh: Mesh,
	rotation: Vec3,
	translation: Vec3,
	screen_width: u32,
	screen_height: u32,
}

impl MeshScene {

	pub fn new(
		graphics: Graphics,
		filepath: String) -> Self
	{

		let mesh = get_mesh_from_obj(filepath);

		let scale = Vec3{
			x: 0.5,
			y: 0.5,
			z: 0.5,
		};

		let rotation = Vec3{
			x: 0.0,
			y: 0.0,
			z: 0.0,
		};

		let translation = Vec3{
			x: 0.0,
			y: -1.0,
			z: 10.0,
		};

		let mesh_color = Vec3{
			x: 0.5,
			y: 0.0,
			z: 0.65,
		};

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

		let pipeline = Pipeline::new(
			graphics,
			DefaultEffect::new(
				scale,
				Mat3::new(),
				translation,
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
			rotation,
			translation,
			screen_width,
			screen_height,
		};

	}

}

impl Scene for MeshScene {

	fn update(&mut self, keyboard_state: &KeyboardState, mouse_state: &MouseState, delta_t_seconds: f32) -> () {

		for keycode in &keyboard_state.keys_pressed {
			match keycode {
				Keycode::Down|Keycode::S { .. } => {
					self.translation.y += 0.1;
					self.pipeline.effect.set_translation(self.translation);
				},
				Keycode::Up|Keycode::W { .. } => {
					self.translation.y -= 0.1;
					self.pipeline.effect.set_translation(self.translation);
				},
				Keycode::Right|Keycode::D { .. } => {
					self.translation.x -= 0.1;
					self.pipeline.effect.set_translation(self.translation);
				},
				Keycode::Left|Keycode::A { .. } => {
					self.translation.x += 0.1;
					self.pipeline.effect.set_translation(self.translation);
				},
				Keycode::Q { .. } => {
					self.translation.z += 0.1;
					self.pipeline.effect.set_translation(self.translation);
				},
				Keycode::E { .. } => {
					self.translation.z -= 0.1;
					self.pipeline.effect.set_translation(self.translation);
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
					self.translation.z += (mouse_state.wheel_y as f32) / 4.0;
					self.pipeline.effect.set_translation(self.translation);
				},
				_ => {}
			}
		}

		let mouse_position = mouse_state.pos.to_owned();

		let nds_mouse_x = mouse_position.0 as f32 / self.screen_width as f32;
		let nds_mouse_y = mouse_position.1 as f32 / self.screen_height as f32;

		// Rotation via mouse input

		// self.rotation.y = -2.0 * PI * nds_mouse_x;
		// self.rotation.x = PI + 2.0 * PI * nds_mouse_y;

		// Rotation via time delta

		// self.rotation.z += 0.2 * PI * delta_t_seconds;
		// self.rotation.z %= 2.0 * PI;

		// self.rotation.x += 0.2 * PI * delta_t_seconds;
		// self.rotation.x %= 2.0 * PI;

		self.rotation.y += 0.2 * PI * delta_t_seconds;
		self.rotation.y %= 2.0 * PI;

		let rotation_matrix =
			Mat3::rotation_x(self.rotation.x) *
			Mat3::rotation_y(self.rotation.y) *
			Mat3::rotation_z(self.rotation.z);

		self.pipeline.effect.set_rotation(rotation_matrix);

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

		let mut point_light_position = Vec3{
			x: 0.0,
			y: 0.0,
			z: 2.0,
		};

		point_light_position.y = 5.0 - (10.0 * nds_mouse_y);
		point_light_position.x = -5.0 + (10.0 * nds_mouse_x);

		self.pipeline.effect.set_point_light_position(
			point_light_position
		);

	}

	fn render(&mut self) -> () {

		self.pipeline.render(&self.mesh);

	}

	fn get_pixel_data(&self) -> &Vec<u32> {

		return self.pipeline.get_pixel_data();

	}

}
