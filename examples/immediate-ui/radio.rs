use cairo::{
    buffer::Buffer2D,
    graphics::Graphics,
    resource::handle::Handle,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        extent::ScreenExtent,
        ui_box::{
            tree::UIBoxTree,
            utils::{container, spacer, text},
            UIBox, UIBoxFeatureFlag, UILayoutDirection,
        },
        UISize, UISizeWithStrictness,
    },
};

use crate::stack::stack;

pub struct RadioOption {
    pub label: String,
}

pub fn radio_group(
    id: String,
    options: &[RadioOption],
    selected_index: usize,
    tree: &mut UIBoxTree,
) -> Result<Option<usize>, String> {
    let mut result: Option<usize> = None;

    tree.with_parent(
        container(
            format!("{}.radio_group.container", id).to_string(),
            UILayoutDirection::TopToBottom,
            None,
        ),
        |tree| -> Result<(), String> {
            stack(
                options,
                |index: usize, option: &RadioOption, tree: &mut UIBoxTree| -> Result<(), String> {
                    if let Some(new_selected_index) =
                        radio(&id, index, option, index == selected_index, tree)?
                    {
                        if new_selected_index != selected_index {
                            result.replace(new_selected_index);
                        }
                    }

                    Ok(())
                },
                4,
                tree,
            )
        },
    )?;

    Ok(result)
}

static RADIO_SELECTED_INDICATOR_SIZE: u32 = 8;

static RADIO_SIZE: u32 = 4 + RADIO_SELECTED_INDICATOR_SIZE + 4;

static RADIO_UI_SIZE: UISizeWithStrictness = UISizeWithStrictness {
    size: UISize::Pixels(RADIO_SIZE),
    strictness: 1.0,
};

fn render_selected_indicator(
    _: &Option<Handle>,
    extent: &ScreenExtent,
    target: &mut Buffer2D,
) -> Result<(), String> {
    GLOBAL_UI_CONTEXT.with(|ctx| {
        let theme = ctx.theme.borrow();

        Graphics::circle(
            target,
            extent.left + 8,
            extent.top + 8,
            5,
            Some(&theme.checkbox_background_selected),
            Some(&theme.checkbox_background_selected),
        );
    });

    Ok(())
}

pub fn radio(
    id: &String,
    index: usize,
    option: &RadioOption,
    is_selected: bool,
    tree: &mut UIBoxTree,
) -> Result<Option<usize>, String> {
    let radio_option_container = {
        let mut ui_box = container(
            format!("{}.radio_option_{}_container", id, index).to_string(),
            UILayoutDirection::LeftToRight,
            None,
        );

        ui_box.features |= UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable;

        ui_box
    };

    let was_radio_option_clicked = tree
        .with_parent(radio_option_container, |tree| {
            let radio_option_selected_indicator = {
                GLOBAL_UI_CONTEXT.with(|ctx| -> Result<UIBox, String> {
                    let theme = ctx.theme.borrow();

                    let fill_color = if is_selected {
                        theme.checkbox_background_selected
                    } else {
                        theme.checkbox_background
                    };

                    let border_color = if is_selected {
                        theme.checkbox_background_selected
                    } else {
                        theme.checkbox_background
                    };

                    ctx.fill_color(fill_color, || {
                        ctx.border_color(border_color, || -> Result<UIBox, String> {
                            Ok(UIBox::new(
                                format!("{}.radio_option_{}_selected", id, index).to_string(),
                                UIBoxFeatureFlag::DrawBorder | UIBoxFeatureFlag::MaskCircle,
                                UILayoutDirection::LeftToRight,
                                [RADIO_UI_SIZE, RADIO_UI_SIZE],
                                if is_selected {
                                    Some((render_selected_indicator, None))
                                } else {
                                    None
                                },
                            ))
                        })
                    })
                })
            }?;

            tree.push(radio_option_selected_indicator)?;

            let radio_option_label = text(
                format!("{}.radio_option_{}_label", id, index).to_string(),
                option.label.to_string(),
            );

            tree.push(spacer(6))?;

            tree.push(radio_option_label)?;

            Ok(())
        })?
        .mouse_interaction_in_bounds
        .was_left_pressed;

    Ok(if was_radio_option_clicked {
        Some(index)
    } else {
        None
    })
}
