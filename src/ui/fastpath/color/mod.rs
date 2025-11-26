use crate::{
    ui::ui_box::{UILayoutDirection, tree::UIBoxTree},
    vec::vec3::Vec3,
};

use super::{
    container::container,
    slider::{SliderOptions, slider},
};

pub fn color_picker(
    id: String,
    color: Vec3,
    options: SliderOptions,
    tree: &mut UIBoxTree,
) -> Result<Option<Vec3>, String> {
    let mut result = None;

    tree.with_parent(
        container(
            format!("{}.color_container", id),
            UILayoutDirection::TopToBottom,
            None,
        ),
        |tree| {
            if let Some(new_red) = slider(format!("{}.red_slider", id), color.x, options, tree)? {
                if result.is_none() {
                    result = Some(color);
                }

                result.as_mut().unwrap().x = new_red;
            }

            if let Some(new_green) = slider(format!("{}.green_slider", id), color.y, options, tree)?
            {
                if result.is_none() {
                    result = Some(color);
                }

                result.as_mut().unwrap().y = new_green;
            }

            if let Some(new_blue) = slider(format!("{}.blue_slider", id), color.z, options, tree)? {
                if result.is_none() {
                    result = Some(color);
                }

                result.as_mut().unwrap().z = new_blue;
            }

            Ok(())
        },
    )?;

    Ok(result)
}
