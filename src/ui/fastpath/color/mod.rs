use crate::{
    color::Color,
    ui::ui_box::{tree::UIBoxTree, UILayoutDirection},
};

use super::{
    container::container,
    slider::{slider, SliderOptions},
};

pub fn color_picker(
    id: String,
    color: Color,
    options: SliderOptions,
    tree: &mut UIBoxTree,
) -> Result<Option<Color>, String> {
    let mut result = None;

    tree.with_parent(
        container(
            format!("{}.color_container", id),
            UILayoutDirection::TopToBottom,
            None,
        ),
        |tree| {
            if let Some(new_red) = slider(format!("{}.red_slider", id), color.r, options, tree)? {
                if result.is_none() {
                    result = Some(color);
                }

                result.as_mut().unwrap().r = new_red;
            }

            if let Some(new_green) = slider(format!("{}.green_slider", id), color.g, options, tree)?
            {
                if result.is_none() {
                    result = Some(color);
                }

                result.as_mut().unwrap().g = new_green;
            }

            if let Some(new_blue) = slider(format!("{}.blue_slider", id), color.b, options, tree)? {
                if result.is_none() {
                    result = Some(color);
                }

                result.as_mut().unwrap().b = new_blue;
            }

            Ok(())
        },
    )?;

    Ok(result)
}
