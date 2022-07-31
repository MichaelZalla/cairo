
use sdl2::{keyboard::Keycode, mouse::MouseWheelDirection};

pub struct KeyboardState {
	pub keys_pressed: Vec<Keycode>,
}

impl KeyboardState {

	pub fn new() -> Self {
		return KeyboardState {
			keys_pressed: vec![],
		};
	}

}

pub struct MouseState {
	pub position: (i32, i32),
	pub wheel_did_move: bool,
	pub wheel_y: i32,
	pub wheel_direction: MouseWheelDirection,
}

impl MouseState {

	pub fn new() -> Self {
		return MouseState {
			position: (0,0),
			wheel_did_move: false,
			wheel_y: 0,
			wheel_direction: MouseWheelDirection::Unknown(0),
		};
	}

}