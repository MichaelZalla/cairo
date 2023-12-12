use super::{
	device::{
		MouseState,
		KeyboardState,
		GameControllerState,
	},
};

pub trait Scene:
{
	fn update(
		&mut self,
		keyboard_state: &KeyboardState,
		mouse_state: &MouseState,
		game_controller_state: &GameControllerState,
		delta_t_seconds: f32) -> ();

	fn render(&mut self) -> ();
	fn get_pixel_data(&self) -> &Vec<u32>;
}
