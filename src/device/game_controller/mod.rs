use core::fmt;

use sdl2::{
    controller::{Axis, Button},
    haptic::Haptic,
};

#[derive(Default)]
pub struct GameController {
    pub id: u32,
    pub name: String,
    pub state: GameControllerState,
    handle: Option<sdl2::controller::GameController>,
    haptic: Option<sdl2::haptic::Haptic>,
}

impl fmt::Debug for GameController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameController")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("handle", &"Unknown")
            .field("haptic", &"Unknown")
            .finish()
    }
}

impl Clone for GameController {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            state: self.state,
            handle: None,
            haptic: None,
        }
    }
}

impl GameController {
    pub fn new() -> Self {
        let mut result: GameController = Default::default();

        result.state.axis_dead_zone = 8000;

        result
    }

    pub fn new_with_handle(handle: sdl2::controller::GameController) -> Self {
        let mut result = GameController::new();

        result.handle = Some(handle);

        result
    }

    pub fn set_button_state(&mut self, button: Button, on: bool) {
        match button {
            Button::A => {
                self.state.buttons.a = on;
            }
            Button::B => {
                self.state.buttons.b = on;
            }
            Button::X => {
                self.state.buttons.x = on;
            }
            Button::Y => {
                self.state.buttons.y = on;
            }
            Button::Back => {
                self.state.buttons.back = on;
            }
            Button::Guide => {
                self.state.buttons.guide = on;
            }
            Button::Start => {
                self.state.buttons.start = on;
            }
            Button::LeftStick => {
                self.state.buttons.left_stick = on;
            }
            Button::RightStick => {
                self.state.buttons.right_stick = on;
            }
            Button::LeftShoulder => {
                self.state.buttons.left_shoulder = on;
            }
            Button::RightShoulder => {
                self.state.buttons.right_shoulder = on;
            }
            Button::DPadUp => {
                self.state.buttons.dpad_up = on;
            }
            Button::DPadDown => {
                self.state.buttons.dpad_down = on;
            }
            Button::DPadLeft => {
                self.state.buttons.dpad_left = on;
            }
            Button::DPadRight => {
                self.state.buttons.dpad_right = on;
            }
            _ => {}
        }
    }

    pub fn set_joystick_state(&mut self, axis: Axis, value: i16) {
        let mut deadzoned_value: i16 = value;

        if (value < 0 && value >= -self.state.axis_dead_zone)
            || (value > 0 && value <= self.state.axis_dead_zone)
        {
            deadzoned_value = 0;
        }

        match axis {
            Axis::LeftX => {
                self.state.joysticks.left.position.x = deadzoned_value;
            }
            Axis::LeftY => {
                self.state.joysticks.left.position.y = deadzoned_value;
            }
            Axis::RightX => {
                self.state.joysticks.right.position.x = deadzoned_value;
            }
            Axis::RightY => {
                self.state.joysticks.right.position.y = deadzoned_value;
            }
            Axis::TriggerLeft => {
                self.state.triggers.left.activation = deadzoned_value;
            }
            Axis::TriggerRight => {
                self.state.triggers.right.activation = deadzoned_value;
            }
        }
    }

    pub fn set_haptic_device(&mut self, device: Haptic) {
        self.haptic = Some(device);
    }

    pub fn set_haptic_intensity(
        &mut self,
        low_intensity: u16,
        high_intensity: u16,
        duration: u32,
    ) -> Result<(), String> {
        if self.handle.is_some() {
            let handle = self.handle.as_mut().unwrap();

            match handle.set_rumble(low_intensity, high_intensity, duration) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!(
                    "Failed to set haptic intensity for device {}: {}",
                    self.id, e
                )),
            }
        } else {
            Err(String::from(
                "Called GameController::set_haptic_intensity with no device handle attached!",
            ))
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerStateButtons {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub back: bool,
    pub guide: bool,
    pub start: bool,
    pub left_stick: bool,
    pub right_stick: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerStateTrigger {
    pub activation: i16,
}
#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerStateTriggers {
    pub left: GameControllerStateTrigger,
    pub right: GameControllerStateTrigger,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerStateJoystickPosition {
    pub x: i16,
    pub y: i16,
}

impl fmt::Display for GameControllerStateJoystickPosition {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            v,
            "GameControllerStateJoystickPosition (x={},y={})",
            self.x, self.y
        )
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerStateJoystick {
    pub position: GameControllerStateJoystickPosition,
}
#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerStateJoysticks {
    pub left: GameControllerStateJoystick,
    pub right: GameControllerStateJoystick,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct GameControllerState {
    pub is_initialized: bool,
    pub axis_dead_zone: i16,
    pub buttons: GameControllerStateButtons,
    pub triggers: GameControllerStateTriggers,
    pub joysticks: GameControllerStateJoysticks,
}

impl fmt::Display for GameController {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(v, "GameController {} ({})", self.id, self.name)
    }
}
