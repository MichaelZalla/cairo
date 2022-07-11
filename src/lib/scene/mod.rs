use super::draw::PixelBuffer;

use super::device::{KeyboardState, MouseState};

pub trait Scene:
{
	fn update(&mut self, keyboard_state: KeyboardState, mouse_state: MouseState, delta_t_seconds: f32) -> ();
	fn render(&mut self, pixel_buffer: &mut PixelBuffer) -> ();
}
