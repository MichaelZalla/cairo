use crate::color::{self, Color};

#[derive(Debug)]
pub struct UITheme {
    // Text
    pub text: Color,
    pub text_hover: Color,
    pub text_pressed: Color,
    pub text_focus: Color,

    // Panel
    pub panel_background: Color,
    pub panel_titlebar_background: Color,
    pub panel_border: Color,

    // Selection state
    pub background_selected: Color,

    // Button
    pub button_background: Color,

    // Text input (?)
    pub input_background: Color,
    pub input_background_slider_alpha: Color,
    pub input_text: Color,
    pub input_cursor: Color,

    // Checkbox
    pub checkbox_background: Color,

    // Dropdown
    pub dropdown_background: Color,

    // Separator
    pub separator: Color,
}

impl Default for UITheme {
    fn default() -> Self {
        // The default theme.
        Self {
            // Text
            text: color::WHITE,
            text_hover: color::WHITE,
            text_pressed: color::GREEN,
            text_focus: color::WHITE,

            // Panel
            panel_background: Color::rgba(56, 56, 56, 220),
            panel_titlebar_background: Color::rgba(45, 45, 45, 220),
            panel_border: Color::rgba(35, 35, 35, 220),

            // Selection state
            background_selected: Color::rgb(71, 114, 179),

            // Button
            button_background: Color::rgb(45, 45, 45),

            // Text input (?)
            input_background: Color::rgb(90, 90, 90),
            input_background_slider_alpha: Color::rgb(69, 69, 69),
            input_text: color::WHITE,
            input_cursor: Color::rgb(124, 124, 124),

            // Checkbox
            checkbox_background: Color::rgb(84, 84, 84),

            // Dropdown
            dropdown_background: Color::rgb(40, 40, 40),

            // Separator
            separator: color::WHITE,
        }
    }
}
