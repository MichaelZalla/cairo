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

#[derive(Debug, Clone)]
pub struct MouseWheelEvent {
    pub direction: MouseWheelDirection,
    pub delta: i32,
}

#[derive(Default, Debug, Clone)]
pub struct MouseState {
    pub buttons_down: HashSet<MouseButton>,
    pub prev_buttons_down: HashSet<MouseButton>,
    pub button_event: Option<MouseEvent>,
    pub position: (i32, i32),
    pub relative_motion: (i32, i32),
    pub ndc_position: (f32, f32),
    pub prev_position: (i32, i32),
    pub prev_ndc_position: (f32, f32),
    pub wheel_event: Option<MouseWheelEvent>,
}
