use sdl2::keyboard::Keycode;

pub mod keycode;

#[derive(Default, Debug, Clone)]
pub struct KeyboardState {
    pub keys_pressed: Vec<Keycode>,
}
