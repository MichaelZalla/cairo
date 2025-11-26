use crate::{
    buffer::Buffer2D,
    color,
    graphics::Graphics,
    resource::handle::Handle,
    ui::{
        UISize, UISizeWithStrictness,
        context::GLOBAL_UI_CONTEXT,
        extent::ScreenExtent,
        fastpath::{container::container, spacer::spacer, stack::stack, text::text},
        ui_box::{UIBox, UIBoxFeatureFlags, UILayoutDirection, tree::UIBoxTree},
    },
    vec::vec2::Vec2,
};

pub struct Checkbox {
    pub value: String,
    pub label: String,
    pub is_checked: bool,
}

impl Checkbox {
    pub fn new(value: &str, label: &str, is_checked: bool) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            is_checked,
        }
    }
}

pub fn checkbox_group(
    id: String,
    items: &[Checkbox],
    tree: &mut UIBoxTree,
) -> Result<Vec<usize>, String> {
    let mut toggled_indices: Vec<usize> = vec![];

    tree.with_parent(
        container(
            format!("{}.checkbox_group.container", id).to_string(),
            UILayoutDirection::TopToBottom,
            None,
        ),
        |tree| -> Result<(), String> {
            stack(
                items,
                |index: usize, item: &Checkbox, tree: &mut UIBoxTree| -> Result<(), String> {
                    if checkbox(&id, index, item, tree)? {
                        toggled_indices.push(index);
                    }

                    Ok(())
                },
                4,
                tree,
            )
        },
    )?;

    Ok(toggled_indices)
}

static CHECKMARK_SIZE: u32 = 9;

static CHECKBOX_SIZE: u32 = 2 + CHECKMARK_SIZE + 2;

static CHECKBOX_UI_SIZE: UISizeWithStrictness = UISizeWithStrictness {
    size: UISize::Pixels(CHECKBOX_SIZE),
    strictness: 1.0,
};

fn render_checkmark(
    _: &Option<Handle>,
    extent: &ScreenExtent,
    target: &mut Buffer2D,
) -> Result<(), String> {
    static OFFSET: Vec2 = Vec2 {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    };

    let mut points: Vec<Vec2> = [
        Vec2 {
            x: extent.left as f32,
            y: (extent.top + 4) as f32,
            z: 0.0,
        },
        Vec2 {
            x: (extent.left + 4) as f32,
            y: (extent.top + 8) as f32,
            z: 0.0,
        },
        Vec2 {
            x: (extent.left + 8) as f32,
            y: extent.top as f32,
            z: 0.0,
        },
    ]
    .iter()
    .map(|p| *p + OFFSET)
    .collect();

    let color_u32 = color::WHITE.to_u32();

    Graphics::poly_line(target, &points, false, color_u32);

    for point in points.iter_mut() {
        point.x += 1.0;
    }

    Graphics::poly_line(target, &points, false, color_u32);

    Ok(())
}

pub fn checkbox(
    id: &String,
    index: usize,
    item: &Checkbox,
    tree: &mut UIBoxTree,
) -> Result<bool, String> {
    let checkbox_option_container = {
        let mut ui_box = container(
            format!("{}.checkbox_{}_container", id, index).to_string(),
            UILayoutDirection::LeftToRight,
            None,
        );

        ui_box.features |= UIBoxFeatureFlags::HOVERABLE | UIBoxFeatureFlags::CLICKABLE;

        ui_box
    };

    let was_checkbox_toggled = tree
        .with_parent(checkbox_option_container, |tree| {
            let checkbox_checked_indicator = {
                GLOBAL_UI_CONTEXT.with(|ctx| -> Result<UIBox, String> {
                    let theme = ctx.theme.borrow();

                    let fill_color = if item.is_checked {
                        theme.background_selected
                    } else {
                        theme.checkbox_background
                    };

                    ctx.fill_color(fill_color, || -> Result<UIBox, String> {
                        Ok(UIBox::new(
                            format!("{}.checkbox_{}_checked", id, index).to_string(),
                            UIBoxFeatureFlags::DRAW_FILL | UIBoxFeatureFlags::DRAW_BORDER,
                            UILayoutDirection::LeftToRight,
                            [CHECKBOX_UI_SIZE, CHECKBOX_UI_SIZE],
                            if item.is_checked {
                                Some((render_checkmark, None))
                            } else {
                                None
                            },
                        ))
                    })
                })
            }?;

            tree.push(checkbox_checked_indicator)?;

            let checkbox_label = text(
                format!("{}.checkbox_{}_label", id, index).to_string(),
                item.label.to_string(),
            );

            tree.push(spacer(6))?;

            tree.push(checkbox_label)?;

            Ok(())
        })?
        .mouse_interaction_in_bounds
        .was_left_pressed;

    Ok(was_checkbox_toggled)
}
