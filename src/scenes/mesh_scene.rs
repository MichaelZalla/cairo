use std::f32::consts::PI;

use sdl2::keyboard::Keycode;

use crate::{
	lib::{
		scene::Scene,
		vec::vec3::Vec3,
		mesh::{Mesh, get_mesh_from_obj},
		device::{KeyboardState, MouseState},
		graphics::Graphics,
		pipeline::{Pipeline, PipelineOptions}
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
			x: 0.0,
			y: 0.0,
			z: 1.0
		};

		let pipeline_options = crate::lib::pipeline::PipelineOptions {
			should_render_wireframe: true,
			should_render_shader: true,
			should_render_normals: false,
		};

		let buffer = &graphics.buffer;

		let screen_width = buffer.width;
		let screen_height = buffer.height;

		let mut pipeline = Pipeline::new(
			graphics,
			DefaultEffect::new(
				scale,
				rotation,
				translation,
				mesh_color,
				ambient_light
			),
			pipeline_options
		);

		pipeline.effect.set_scale(scale);
		pipeline.effect.set_rotation(rotation);
		pipeline.effect.set_translation(translation);

		pipeline.set_light_normal(ambient_light.as_normal());

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

		self.rotation.y = -2.0 * PI * nds_mouse_x;
		self.rotation.x = PI + 2.0 * PI * nds_mouse_y;

		// self.rotation.z += 0.2 * PI * delta_t_seconds;
		// self.rotation.z %= 2.0 * PI;

		// self.rotation.x += 0.2 * PI * delta_t_seconds;
		// self.rotation.x %= 2.0 * PI;

		// self.rotation.y += 0.2 * PI * delta_t_seconds;
		// self.rotation.y %= 2.0 * PI;

		self.pipeline.effect.set_rotation(self.rotation);

		// self.pipeline.effect.set_ambient_light(Vec3 {
		// 	x: -1.0 * (mouse_state.pos.0 as f32) / 20.0,
		// 	y: (mouse_state.pos.1 as f32) / 20.0,
		// 	z: 0.0,
		// }.as_normal());

	}

	fn render(&mut self) -> () {

		self.pipeline.render(&self.mesh);

	}

	fn get_pixel_data(&self) -> &Vec<u32> {

		return self.pipeline.get_pixel_data();

	}

}
