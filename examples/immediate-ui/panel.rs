use std::fmt::Debug;

use cairo::{
    serde::PostDeserialize,
    ui::ui_box::{
        tree::UIBoxTree,
        utils::{button_box, text_box},
    },
};
use serde::{Deserialize, Serialize};

pub trait PanelInstance {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String>;
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct ButtonPanel {
    id: String,
    clicked_count: usize,
}

impl ButtonPanel {
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            clicked_count: 0,
        }
    }
}

impl Debug for ButtonPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonPanel")
            .field("clicked_count", &self.clicked_count)
            .finish()
    }
}

impl PostDeserialize for ButtonPanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for ButtonPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        if tree
            .push(button_box(
                format!("button_{}", self.id).to_string(),
                format!("Button {}", self.id).to_string(),
                None,
            ))?
            .mouse_interaction_in_bounds
            .was_left_pressed
        {
            self.clicked_count += 1;
        }

        tree.push(text_box(
            format!("button_clicked_text{}", self.id).to_string(),
            format!("Clicked {} times!", self.clicked_count).to_string(),
        ))?;

        Ok(())
    }
}
