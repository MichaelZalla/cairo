use core::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::color::{self, Color};

use super::{
    context::{UIContext, GLOBAL_UI_CONTEXT},
    ui_box::{
        interaction::UIBoxInteraction,
        tree::{UIBoxTree, UIBoxTreeRenderCallback},
        utils::text_box,
        UIBox, UIBoxFeatureFlag, UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

pub mod tree;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PanelMetadata<T> {
    pub panel_type: T,
    #[serde(skip)]
    pub render_callback: Option<UIBoxTreeRenderCallback>,
}

impl<T: fmt::Debug> fmt::Debug for PanelMetadata<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PanelMetadata")
            .field("panel_type", &self.panel_type)
            .field(
                "render_callback",
                match self.render_callback {
                    Some(_) => &"Some(Rc<dyn Fn(&mut UIBoxTree) -> Result<(), String>>)",
                    None => &"None ",
                },
            )
            .finish()
    }
}

impl<T: fmt::Debug> fmt::Display for PanelMetadata<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Panel<T> {
    pub path: String,
    // For this panel.
    pub resizable: bool,
    pub alpha_split: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panel_metadata: Option<PanelMetadata<T>>,
    // For child panels.
    pub layout_direction: UILayoutDirection,
}

impl<'a, T: Default + Clone + fmt::Debug + Display + Serialize + Deserialize<'a>> fmt::Display
    for Panel<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Panel ({})", self.path)
    }
}

impl<'a, T: Default + Clone + fmt::Debug + Display + Serialize + Deserialize<'a>> Panel<T> {
    pub fn new(
        alpha_split: f32,
        panel_metadata: Option<PanelMetadata<T>>,
        layout_direction: UILayoutDirection,
    ) -> Self {
        Self {
            path: "".to_string(),
            resizable: true,
            alpha_split,
            panel_metadata,
            layout_direction,
        }
    }

    fn get_panel_ui_box_id_and_hash(&self) -> (String, String) {
        let panel_path = &self.path;
        let panel_path_cloned = panel_path.clone();
        let panel_path_components = panel_path_cloned.split(' ').collect::<Vec<&str>>();

        let panel_ui_box_id = panel_path_components.join("");
        let panel_ui_box_key_hash = panel_path_components
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>()
            .join("_");

        (panel_ui_box_id, panel_ui_box_key_hash)
    }

    pub fn make_panel_box(&self, ui_context: &UIContext<'static>) -> Result<UIBox, String> {
        let (panel_ui_box_id, panel_ui_box_key_hash) = self.get_panel_ui_box_id_and_hash();

        let ui_box_feature_flags = UIBoxFeatureFlag::DrawFill
            | UIBoxFeatureFlag::Hoverable
            | UIBoxFeatureFlag::Clickable
            | if self.resizable {
                UIBoxFeatureFlag::Resizable
            } else {
                UIBoxFeatureFlag::Null
            };

        let mut panel_ui_box: UIBox = Default::default();

        ui_context.fill_color(color::WHITE, || {
            ui_context.border_color(color::BLACK, || {
                panel_ui_box = UIBox::new(
                    format!("{}__{}", panel_ui_box_id, panel_ui_box_key_hash),
                    ui_box_feature_flags,
                    self.layout_direction,
                    [
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(self.alpha_split),
                            strictness: 0.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(1.0),
                            strictness: 1.0,
                        },
                    ],
                );

                Ok(())
            })
        })?;

        Ok(panel_ui_box)
    }

    pub fn render_leaf_panel_contents(
        &self,
        ui_box_tree: &mut UIBoxTree,
        panel_interaction_result: &UIBoxInteraction,
    ) -> Result<(), String> {
        match &self.panel_metadata {
            Some(metadata) => match &metadata.render_callback {
                Some(render) => render(ui_box_tree),
                None => {
                    let _result = ui_box_tree
                        .push(text_box(String::new(), format!("{}", metadata.panel_type)))?;

                    Ok(())
                }
            },
            None => render_debug_interaction_result(ui_box_tree, panel_interaction_result),
        }
    }
}

fn render_debug_interaction_result(
    ui_box_tree: &mut UIBoxTree,
    interaction_result: &UIBoxInteraction,
) -> Result<(), String> {
    // Push some text describing this leaf panel's interaction.

    let mouse_result = &interaction_result.mouse_interaction_in_bounds;

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.text_color(Color::rgb(165, 165, 165), || {
            ui_box_tree.push(text_box(
                String::new(),
                format!("is_hovering: {}", mouse_result.is_hovering),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("was_left_pressed: {}", mouse_result.was_left_pressed),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("is_left_down: {}", mouse_result.is_left_down),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("was_left_released: {}", mouse_result.was_left_released),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("was_middle_pressed: {}", mouse_result.was_middle_pressed),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("is_middle_down: {}", mouse_result.is_middle_down),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("was_middle_released: {}", mouse_result.was_middle_released),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("was_right_pressed: {}", mouse_result.was_right_pressed),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("is_right_down: {}", mouse_result.is_right_down),
            ))?;

            ui_box_tree.push(text_box(
                String::new(),
                format!("was_right_released: {}", mouse_result.was_right_released),
            ))?;

            Ok(())
        })
        .unwrap();
    });

    Ok(())
}
