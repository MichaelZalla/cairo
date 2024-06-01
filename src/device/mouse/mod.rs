use std::collections::HashSet;

use sdl2::mouse::{MouseButton, MouseWheelDirection};

pub mod cursor;

#[derive(Default, Copy, Clone, Debug)]
pub enum MouseEventKind {
    #[default]
    Down,
    Up,
}

#[derive(Copy, Clone, Debug)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub kind: MouseEventKind,
}

impl Default for MouseEvent {
    fn default() -> Self {
        MouseEvent {
            button: MouseButton::Unknown,
            kind: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MouseState {
    pub buttons_down: HashSet<MouseButton>,
    pub prev_buttons_down: HashSet<MouseButton>,
    pub button_event: Option<MouseEvent>,
    pub position: (i32, i32),
    pub relative_motion: (i32, i32),
    pub ndc_position: (f32, f32),
    pub prev_position: (i32, i32),
    pub prev_ndc_position: (f32, f32),
    pub wheel_did_move: bool,
    pub wheel_y: i32,
    pub wheel_direction: MouseWheelDirection,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            buttons_down: Default::default(),
            prev_buttons_down: Default::default(),
            button_event: None,
            position: (0, 0),
            ndc_position: (0.0, 0.0),
            prev_position: (0, 0),
            prev_ndc_position: (0.0, 0.0),
            relative_motion: (0, 0),
            wheel_did_move: false,
            wheel_y: 0,
            wheel_direction: MouseWheelDirection::Unknown(0),
        }
    }
}
