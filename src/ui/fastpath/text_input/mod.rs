use sdl2::keyboard::Keycode;

use crate::{
    device::keyboard::keycode,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{tree::UIBoxTree, UIBox, UIBoxFeatureFlag, UILayoutDirection},
        UISize, UISizeWithStrictness,
    },
};

pub fn text_input(
    id: String,
    value: &String,
    tree: &mut UIBoxTree,
) -> Result<Option<String>, String> {
    let mut result: Option<String> = None;

    let mut text_input_box = GLOBAL_UI_CONTEXT.with(|ctx| -> Result<UIBox, String> {
        let theme = ctx.theme.borrow();

        ctx.fill_color(theme.input_background, || -> Result<UIBox, String> {
            ctx.border_color(theme.panel_border, || -> Result<UIBox, String> {
                Ok(UIBox::new(
                    id,
                    UIBoxFeatureFlag::Hoverable
                        | UIBoxFeatureFlag::Clickable
                        | UIBoxFeatureFlag::DrawFill
                        | UIBoxFeatureFlag::DrawBorder
                        | UIBoxFeatureFlag::DrawText,
                    UILayoutDirection::LeftToRight,
                    [
                        UISizeWithStrictness {
                            size: UISize::Pixels(20),
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::Pixels(150),
                            strictness: 1.0,
                        },
                    ],
                    None,
                ))
            })
        })
    })?;

    text_input_box.text_content = Some(value.clone());

    let interaction_result = tree.push(text_input_box)?;

    if interaction_result.was_focused {
        GLOBAL_UI_CONTEXT.with(|ctx| {
            let input_events = ctx.input_events.borrow();

            let pressed_keys = &input_events.keyboard.newly_pressed_keycodes;

            for keycode in pressed_keys {
                match *keycode {
                    Keycode::BACKSPACE | Keycode::Delete => {
                        // Remove one character from the model value, if possible.

                        let mut new_value = value.clone();
                        let _ = new_value.pop();

                        result.replace(new_value);
                    }
                    _ => {
                        match keycode::to_ascii_char(keycode, pressed_keys) {
                            Some(char) => {
                                // Add this character to the model value (string).

                                let new_value = format!("{}{}", value, char);

                                result.replace(new_value);
                            }
                            None => {
                                println!("Unsupported keycode: {}", keycode);

                                // Ignore this keypress.
                            }
                        }
                    }
                }
            }
        });
    }

    Ok(result)
}
