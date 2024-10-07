use std::fmt::Debug;

use cairo::{
    serde::PostDeserialize,
    ui::ui_box::{
        tree::UIBoxTree,
        utils::{button_box, text_box},
        UIBoxFeatureFlag,
    },
};

use crate::{COMMAND_BUFFER, SETTINGS};

pub trait PanelInstance {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String>;
}

#[derive(Clone)]
pub(crate) struct SettingsPanel {
    id: String,
}

impl SettingsPanel {
    pub fn from_id(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl Debug for SettingsPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsPanel")
            .field("id", &self.id)
            .finish()
    }
}

impl PostDeserialize for SettingsPanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for SettingsPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        let clicked_count = SETTINGS.with(|settings| *settings.clicked_count.borrow());

        if tree
            .push(button_box(
                format!("button_{}", self.id).to_string(),
                format!("Button {}", self.id).to_string(),
                None,
            ))?
            .mouse_interaction_in_bounds
            .was_left_pressed
        {
            COMMAND_BUFFER.with(|buffer| {
                let mut pending_queue = buffer.pending_commands.borrow_mut();

                pending_queue.push_back(
                    format!("set_setting clicked_count {}", clicked_count + 1).to_string(),
                );
            });
        }

        let mut dynamic_text_box = text_box(
            format!("button_clicked_text{}", self.id).to_string(),
            format!("settings.click_count: {}", clicked_count).to_string(),
        );

        dynamic_text_box.features |= UIBoxFeatureFlag::SkipTextCaching;

        tree.push(dynamic_text_box)?;

        Ok(())
    }
}
