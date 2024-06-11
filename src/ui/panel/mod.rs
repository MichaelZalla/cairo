use core::fmt;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    color::{self, Color},
    resource::handle::Handle,
};

use super::{
    context::{UIContext, GLOBAL_UI_CONTEXT},
    ui_box::{
        interaction::UIBoxInteraction, tree::UIBoxTree, utils::text_box, UIBox, UIBoxFeatureFlag,
        UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

pub mod tree;

pub type PanelRenderCallback = Rc<dyn Fn(&Handle, &mut UIBoxTree) -> Result<(), String>>;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PanelInstanceData {
    pub panel_instance: Handle,
    #[serde(skip)]
    pub render: Option<PanelRenderCallback>,
}

impl fmt::Debug for PanelInstanceData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PanelInstanceData")
            .field(
                "render",
                match self.render {
                    Some(_) => &"Some(PanelRenderCallback)",
                    None => &"None ",
                },
            )
            .finish()
    }
}
impl fmt::Display for PanelInstanceData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub path: String,
    // For this panel.
    pub resizable: bool,
    pub alpha_split: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_data: Option<PanelInstanceData>,
    // For child panels.
    pub layout_direction: UILayoutDirection,
}

impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Panel ({})", self.path)
    }
}

impl Panel {
    pub fn new(
        alpha_split: f32,
        instance_data: Option<PanelInstanceData>,
        layout_direction: UILayoutDirection,
    ) -> Self {
        Self {
            path: "".to_string(),
            resizable: true,
            alpha_split,
            instance_data,
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

    pub fn make_panel_box(
        &self,
        ui_context: &UIContext<'static>,
        draw_border: bool,
    ) -> Result<UIBox, String> {
        let (panel_ui_box_id, panel_ui_box_key_hash) = self.get_panel_ui_box_id_and_hash();

        let ui_box_feature_flags = UIBoxFeatureFlag::DrawFill
            | if draw_border {
                UIBoxFeatureFlag::DrawBorder
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
        match &self.instance_data {
            Some(instance_data) => match &instance_data.render {
                Some(render) => render(&instance_data.panel_instance, ui_box_tree),
                None => {
                    let _result = ui_box_tree.push(text_box(
                        String::new(),
                        format!("Panel {}", &instance_data.panel_instance.uuid),
                    ))?;

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
