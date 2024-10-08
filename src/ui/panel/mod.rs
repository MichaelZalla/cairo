use core::fmt;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{color::Color, resource::handle::Handle};

use super::{
    context::GLOBAL_UI_CONTEXT,
    ui_box::{
        interaction::UIBoxInteraction, tree::UIBoxTree, utils::text, UIBox,
        UIBoxCustomRenderCallback, UIBoxFeatureFlag, UILayoutDirection,
    },
    window::Window,
    UISize, UISizeWithStrictness,
};

pub mod tree;

pub type PanelRenderCallback = Rc<dyn Fn(&Handle, &mut UIBoxTree) -> Result<(), String>>;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PanelInstanceData {
    pub panel_instance: Handle,
    #[serde(skip)]
    pub render: Option<PanelRenderCallback>,
    #[serde(skip)]
    pub custom_render_callback: Option<Rc<UIBoxCustomRenderCallback>>,
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
            .field(
                "custom_render_callback",
                match self.custom_render_callback {
                    Some(_) => &"Some(Rc<UIBoxCustomRenderCallback>)",
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Panel {
    fn default() -> Self {
        Self {
            path: "unknown".to_string(),
            resizable: true,
            alpha_split: 1.0,
            instance_data: None,
            layout_direction: Default::default(),
        }
    }
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

    fn get_panel_ui_box_id(&self) -> String {
        let panel_path = &self.path;
        let panel_path_cloned = panel_path.clone();
        let panel_path_components = panel_path_cloned.split(' ').collect::<Vec<&str>>();

        panel_path_components.join("")
    }

    pub fn make_panel_box(&self, window: &Window) -> Result<UIBox, String> {
        let panel_ui_box_id = self.get_panel_ui_box_id();

        let draw_border = !window.docked;

        let ui_box_feature_flags = UIBoxFeatureFlag::Null
            | if draw_border {
                UIBoxFeatureFlag::DrawBorder
            } else {
                UIBoxFeatureFlag::Null
            }
            | if let Some(data) = &self.instance_data {
                if data.custom_render_callback.is_some() {
                    UIBoxFeatureFlag::DrawCustomRender
                } else {
                    UIBoxFeatureFlag::Null
                }
            } else {
                UIBoxFeatureFlag::Null
            };

        let panel_ui_box = UIBox::new(
            panel_ui_box_id.to_string(),
            ui_box_feature_flags,
            self.layout_direction,
            [
                if self.alpha_split < 0.999 {
                    UISizeWithStrictness {
                        size: UISize::PercentOfParent(self.alpha_split),
                        strictness: 0.0,
                    }
                } else {
                    UISizeWithStrictness {
                        size: UISize::ChildrenSum,
                        strictness: 1.0,
                    }
                },
                UISizeWithStrictness {
                    size: UISize::ChildrenSum,
                    strictness: 1.0,
                },
            ],
            if let Some(data) = &self.instance_data {
                data.custom_render_callback
                    .as_ref()
                    .map(|callback| (data.panel_instance, callback.clone()))
            } else {
                None
            },
        );

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
                    let _result = ui_box_tree.push(text(
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
            ui_box_tree.push(text(
                String::new(),
                format!("is_hovering: {}", mouse_result.is_hovering),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("was_left_pressed: {}", mouse_result.was_left_pressed),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("is_left_down: {}", mouse_result.is_left_down),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("was_left_released: {}", mouse_result.was_left_released),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("was_middle_pressed: {}", mouse_result.was_middle_pressed),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("is_middle_down: {}", mouse_result.is_middle_down),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("was_middle_released: {}", mouse_result.was_middle_released),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("was_right_pressed: {}", mouse_result.was_right_pressed),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("is_right_down: {}", mouse_result.is_right_down),
            ))?;

            ui_box_tree.push(text(
                String::new(),
                format!("was_right_released: {}", mouse_result.was_right_released),
            ))?;

            Ok(())
        })
        .unwrap();
    });

    Ok(())
}
