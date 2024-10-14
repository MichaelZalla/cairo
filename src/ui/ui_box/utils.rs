use crate::ui::{context::GLOBAL_UI_CONTEXT, UISize, UISizeWithStrictness};

use super::{
    interaction::UIBoxInteraction, key::UIKey, tree::UIBoxTree, UIBox, UIBoxFeatureFlag,
    UIBoxFeatureMask, UILayoutDirection, UI_BOX_SPACER_ID,
};

pub fn container(
    id: String,
    layout_direction: UILayoutDirection,
    semantic_sizes: Option<[UISizeWithStrictness; 2]>,
) -> UIBox {
    let sizes = match semantic_sizes {
        Some(sizes) => sizes,
        None => [
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::ChildrenSum,
                strictness: 1.0,
            },
        ],
    };

    UIBox::new(id, UIBoxFeatureMask::none(), layout_direction, sizes, None)
}

pub fn greedy_container(id: String, layout_direction: UILayoutDirection) -> UIBox {
    UIBox::new(
        id,
        UIBoxFeatureMask::none(),
        layout_direction,
        [
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 0.0,
            },
            UISizeWithStrictness {
                size: UISize::PercentOfParent(1.0),
                strictness: 1.0,
            },
        ],
        None,
    )
}

pub fn text(id: String, label: String) -> UIBox {
    let mut text_box = UIBox::new(
        id,
        UIBoxFeatureFlag::Null | UIBoxFeatureFlag::DrawText,
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
        ],
        None,
    );

    text_box.text_content = Some(label);

    text_box
}

pub fn button(
    id: String,
    label: String,
    semantic_sizes: Option<[UISizeWithStrictness; 2]>,
) -> UIBox {
    let sizes = match semantic_sizes {
        Some(sizes) => sizes,
        None => [
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::TextContent,
                strictness: 1.0,
            },
        ],
    };

    let mut button_box = UIBox::new(
        id,
        UIBoxFeatureFlag::DrawFill
            | UIBoxFeatureFlag::DrawBorder
            | UIBoxFeatureFlag::EmbossAndDeboss
            | UIBoxFeatureFlag::DrawText
            | UIBoxFeatureFlag::Hoverable
            | UIBoxFeatureFlag::Clickable,
        UILayoutDirection::LeftToRight,
        sizes,
        None,
    );

    button_box.text_content = Some(label);

    button_box
}

#[derive(Debug, Copy, Clone)]
pub struct SliderOptions {
    pub min: f32,
    pub max: f32,
    pub decimals: usize,
    pub is_vertical: bool,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            decimals: 2,
            is_vertical: false,
        }
    }
}

pub fn slider(
    id: String,
    value: f32,
    options: SliderOptions,
    tree: &mut UIBoxTree,
) -> Result<Option<f32>, String> {
    let container_id = format!("{}_slider_container", id);

    let ui_key = UIKey::from_string(container_id.clone());

    let interaction_result: UIBoxInteraction =
        GLOBAL_UI_CONTEXT.with(|ctx| -> Result<UIBoxInteraction, String> {
            let theme = ctx.theme.borrow();
            let cache = ctx.cache.borrow();

            let was_dragging = if let Some(entry) = cache.get(&ui_key) {
                entry.active
            } else {
                false
            };

            let fill_color = if was_dragging {
                theme.dropdown_background
            } else {
                theme.input_background
            };

            let slider_container = ctx.fill_color(fill_color, || -> Result<UIBox, String> {
                ctx.border_color(theme.panel_border, || -> Result<UIBox, String> {
                    Ok(UIBox::new(
                        container_id,
                        UIBoxFeatureFlag::DrawFill
                            | UIBoxFeatureFlag::DrawBorder
                            | UIBoxFeatureFlag::Hoverable
                            | UIBoxFeatureFlag::Clickable,
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
            })?;

            tree.with_parent(slider_container, |tree| {
                tree.push(greedy_spacer())?;

                let label = format!("{:.1$}", value, options.decimals);

                tree.push(text(format!("{}_slider_value_label", id), label))?;

                tree.push(greedy_spacer())?;

                Ok(())
            })
        })?;

    let slider_result: Option<f32> = GLOBAL_UI_CONTEXT.with(|ctx| -> Option<f32> {
        let cache = ctx.cache.borrow();

        match (
            interaction_result.mouse_interaction_in_bounds.drag_event,
            cache.get(&ui_key),
        ) {
            (Some(drag_event), Some(entry)) => {
                let (extent, local_drag_delta) = if options.is_vertical {
                    (entry.computed_size[1], drag_event.delta.1 as f32)
                } else {
                    (entry.computed_size[0], drag_event.delta.0 as f32)
                };

                let drag_alpha = local_drag_delta / extent;

                let value_delta = (options.max - options.min) * drag_alpha;

                let new_value = (value + value_delta).max(options.min).min(options.max);

                Some(new_value)
            }
            _ => None,
        }
    });

    Ok(slider_result)
}

pub fn spacer(size: u32) -> UIBox {
    UIBox::new(
        UI_BOX_SPACER_ID.to_string(),
        UIBoxFeatureMask::none(),
        UILayoutDirection::LeftToRight,
        [
            UISizeWithStrictness {
                size: UISize::Pixels(size),
                strictness: 1.0,
            },
            UISizeWithStrictness {
                size: UISize::MaxOfSiblings,
                strictness: 1.0,
            },
        ],
        None,
    )
}

pub fn greedy_spacer() -> UIBox {
    let mut spacer = spacer(0);

    spacer.semantic_sizes[0] = UISizeWithStrictness {
        size: UISize::PercentOfParent(1.0),
        strictness: 0.0,
    };

    spacer
}
