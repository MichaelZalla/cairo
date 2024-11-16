use std::collections::HashSet;

use sdl2::keyboard::{Keycode, Mod};

pub mod keycode;

#[derive(Debug, Clone)]
pub struct KeyboardState {
    pub modifiers: Mod,
    pub pressed_keycodes: HashSet<Keycode>,
    pub newly_pressed_keycodes: HashSet<Keycode>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            modifiers: Mod::NOMOD,
            pressed_keycodes: Default::default(),
            newly_pressed_keycodes: Default::default(),
        }
    }
}
