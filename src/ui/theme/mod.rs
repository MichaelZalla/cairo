use crate::color::{self, Color};

#[derive(Default, Debug)]
pub struct UITheme {
    pub border: Color,
    pub border_hover: Color,
    pub border_pressed: Color,
    pub border_focus: Color,
    pub text: Color,
    pub text_hover: Color,
    pub text_pressed: Color,
    pub text_focus: Color,
}

pub static DEFAULT_UI_THEME: UITheme = UITheme {
    border: color::YELLOW,
    border_hover: color::WHITE,
    border_pressed: color::GREEN,
    border_focus: color::RED,
    text: color::YELLOW,
    text_hover: color::WHITE,
    text_pressed: color::GREEN,
    text_focus: color::RED,
};
