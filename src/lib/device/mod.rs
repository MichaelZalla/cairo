use std::fmt;
use sdl2::{
	mouse::MouseWheelDirection,
	keyboard::Keycode,
	controller::{Button, Axis}
};

#[derive(Clone)]
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

#[derive(Default)]
pub struct GameController {
	pub id: u32,
	pub name: String,
	pub state: GameControllerState,
	handle: Option<sdl2::controller::GameController>,
}

impl GameController {

	pub fn new() -> Self {

		let mut result: GameController = Default::default();

		result.state.axis_dead_zone = 8000;

		return result;

	}

	pub fn new_with_handle(
		handle: sdl2::controller::GameController) -> Self
	{

		let mut result = GameController::new();

		result.handle = Some(handle);

		return result;

	}

	pub fn set_button_state(
		&mut self,
		button: Button,
		on: bool)
	{
		match button {
			Button::A => {
				self.state.buttons.A = on;
			},
			Button::B => {
				self.state.buttons.B = on;
			},
			Button::X => {
				self.state.buttons.X = on;
			},
			Button::Y => {
				self.state.buttons.Y = on;
			},
			Button::Back => {
				self.state.buttons.BACK = on;
			},
			Button::Guide => {
				self.state.buttons.GUIDE = on;
			},
			Button::Start => {
				self.state.buttons.START = on;
			},
			Button::LeftStick => {
				self.state.buttons.LEFT_STICK = on;
			},
			Button::RightStick => {
				self.state.buttons.RIGHT_STICK = on;
			},
			Button::LeftShoulder => {
				self.state.buttons.LEFT_SHOULDER = on;
			},
			Button::RightShoulder => {
				self.state.buttons.RIGHT_SHOULDER = on;
			},
			Button::DPadUp => {
				self.state.buttons.DPAD_UP = on;
			},
			Button::DPadDown => {
				self.state.buttons.DPAD_DOWN = on;
			},
			Button::DPadLeft => {
				self.state.buttons.DPAD_LEFT = on;
			},
			Button::DPadRight => {
				self.state.buttons.DPAD_RIGHT = on;
			},
			_ => {},
		}
	}

	pub fn set_joystick_state(
		&mut self,
		axis: Axis,
		mut value: i16)
	{

		if value.abs() <= self.state.axis_dead_zone {
			value = 0;
		}

		match axis {
			Axis::LeftX => {
				self.state.joysticks.left.position.x = value;
			},
			Axis::LeftY => {
				self.state.joysticks.left.position.y = value;
			},
			Axis::RightX => {
				self.state.joysticks.right.position.x = value;
			},
			Axis::RightY => {
				self.state.joysticks.right.position.y = value;
			},
			Axis::TriggerLeft => {
				self.state.triggers.left.activation = value;
			},
			Axis::TriggerRight => {
				self.state.triggers.right.activation = value;
			},
		}

	}

}

#[derive(Default, Clone)]
pub struct GameControllerStateButtons {
	pub A: bool,
	pub B: bool,
	pub X: bool,
	pub Y: bool,
	pub BACK: bool,
	pub GUIDE: bool,
	pub START: bool,
	pub LEFT_STICK: bool,
	pub RIGHT_STICK: bool,
	pub LEFT_SHOULDER: bool,
	pub RIGHT_SHOULDER: bool,
	pub DPAD_UP: bool,
	pub DPAD_DOWN: bool,
	pub DPAD_LEFT: bool,
	pub DPAD_RIGHT: bool,
}

#[derive(Default, Clone)]
pub struct GameControllerStateTrigger {
	pub activation: i16,
}
#[derive(Default, Clone)]
pub struct GameControllerStateTriggers {
	pub left: GameControllerStateTrigger,
	pub right: GameControllerStateTrigger,
}

#[derive(Default, Clone)]
pub struct GameControllerStateJoystickPosition {
	pub x: i16,
	pub y: i16,
}

impl fmt::Display for GameControllerStateJoystickPosition {
	fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(v, "GameControllerStateJoystickPosition (x={},y={})", self.x, self.y)
	}
}

#[derive(Default, Clone)]
pub struct GameControllerStateJoystick {
	pub position: GameControllerStateJoystickPosition,
}
#[derive(Default, Clone)]
pub struct GameControllerStateJoysticks {
	pub left: GameControllerStateJoystick,
	pub right: GameControllerStateJoystick,
}

#[derive(Default, Clone)]
pub struct GameControllerState {
	pub is_initialized: bool,
	pub axis_dead_zone: i16,
	pub buttons: GameControllerStateButtons,
	pub triggers: GameControllerStateTriggers,
	pub joysticks: GameControllerStateJoysticks,
}

impl<'a> fmt::Display for GameController {
	fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(v, "GameController {} ({})", self.id, self.name)
	}
}
