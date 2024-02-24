use crate::color::{self, Color};

#[derive(Default, Debug)]
pub struct UITheme {
    pub text: Color,
    pub text_hover: Color,
    pub text_pressed: Color,
    pub text_focus: Color,
    pub button_background: Color,
    pub input_background: Color,
    pub input_background_slider_alpha: Color,
    pub input_text: Color,
    pub input_cursor: Color,
    pub checkbox_background: Color,
    pub dropdown_background: Color,
    pub separator: Color,
    pub panel_background: Color,
    pub panel_titlebar_background: Color,
    pub panel_border: Color,
}

pub static DEFAULT_UI_THEME: UITheme = UITheme {
    text: color::WHITE,
    text_hover: color::WHITE,
    text_pressed: color::GREEN,
    text_focus: color::WHITE,
    button_background: Color::rgb(45, 45, 45),
    input_background: Color::rgb(90, 90, 90),
    input_background_slider_alpha: Color::rgb(69, 69, 69),
    input_text: color::WHITE,
    input_cursor: Color::rgb(124, 124, 124),
    checkbox_background: Color::rgb(45, 45, 45),
    dropdown_background: Color::rgb(45, 45, 45),
    separator: color::WHITE,
    panel_background: Color::rgb(56, 56, 56),
    panel_titlebar_background: Color::rgb(45, 45, 45),
    panel_border: Color::rgb(35, 35, 35),
};
