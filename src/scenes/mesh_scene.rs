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
};

pub struct MeshScene {
	pipeline: Pipeline,
	pipeline_options: PipelineOptions,
	mesh: Mesh,
	translation: Vec3,
	rotation: Vec3,
	light_vector: Vec3,
	normalized_light_vector: Vec3,
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

		let light_vector = Vec3{
			x: 0.0,
			y: 0.0,
			z: 1.0
		};

		let normalized_light_vector = Vec3{
			x: 0.0,
			y: 0.0,
			z: 1.0
		};

		let pipeline_options = crate::lib::pipeline::PipelineOptions {
			should_render_wireframe: true,
			should_render_shader: true,
			should_render_normals: false,
		};

		let mut pipeline = Pipeline::new(graphics, pipeline_options);

		pipeline.set_scale(scale);
		pipeline.set_rotation(rotation);
		pipeline.set_translation(translation);

		pipeline.set_light_normal(normalized_light_vector);


		return MeshScene{
			pipeline: pipeline,
			pipeline_options: pipeline_options,
			mesh: mesh,
			rotation: rotation,
			translation: translation,
			light_vector: light_vector,
			normalized_light_vector: normalized_light_vector,
		};

	}

}

impl Scene for MeshScene {

	fn update(&mut self, keyboard_state: &KeyboardState, mouse_state: &MouseState, delta_t_seconds: f32) -> () {

		for keycode in &keyboard_state.keys_pressed {
			match keycode {
				Keycode::Down|Keycode::S { .. } => {
					self.translation.y += 0.1;
					self.pipeline.set_translation(self.translation);
				},
				Keycode::Up|Keycode::W { .. } => {
					self.translation.y -= 0.1;
					self.pipeline.set_translation(self.translation);
				},
				Keycode::Right|Keycode::D { .. } => {
					self.translation.x -= 0.1;
					self.pipeline.set_translation(self.translation);
				},
				Keycode::Left|Keycode::A { .. } => {
					self.translation.x += 0.1;
					self.pipeline.set_translation(self.translation);
				},
				Keycode::Q { .. } => {
					self.translation.z += 0.1;
					self.pipeline.set_translation(self.translation);
				},
				Keycode::E { .. } => {
					self.translation.z -= 0.1;
					self.pipeline.set_translation(self.translation);
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
					self.pipeline.set_translation(self.translation);
				},
				_ => {}
			}
		}

		self.light_vector.x = -1.0 * (mouse_state.pos.0 as f32) / 20.0;
		self.light_vector.y = (mouse_state.pos.1 as f32) / 20.0;

		self.normalized_light_vector = self.light_vector.as_normal();

		self.pipeline.set_light_normal(self.normalized_light_vector);

		self.rotation.z += 0.25 * PI * delta_t_seconds;
		self.rotation.z %= 2.0 * PI;

		self.rotation.x += 0.25 * PI * delta_t_seconds;
		self.rotation.x %= 2.0 * PI;

		self.rotation.y += 0.25 * PI * delta_t_seconds;
		self.rotation.y %= 2.0 * PI;

		self.pipeline.set_rotation(self.rotation);

	}

	fn render(&mut self) -> () {

		self.pipeline.render(&self.mesh);

	}

	fn get_pixel_data(&self) -> &Vec<u32> {

		return self.pipeline.get_pixel_data();

	}

}
