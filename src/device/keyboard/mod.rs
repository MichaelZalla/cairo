use sdl2::keyboard::{Keycode, Mod};

pub mod keycode;

#[derive(Default, Debug, Clone)]
pub struct KeyboardState {
    pub keys_pressed: Vec<(Keycode, Mod)>,
}
