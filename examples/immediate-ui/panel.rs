use std::fmt::Debug;

use cairo::{
    serde::PostDeserialize,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{
            tree::UIBoxTree,
            utils::{button, container, spacer, text},
            UIBox, UIBoxFeatureFlag, UILayoutDirection,
        },
    },
};

use crate::{COMMAND_BUFFER, SETTINGS};

pub trait PanelInstance {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String>;
}

#[derive(Clone)]
pub(crate) struct SettingsPanel {
    id: String,
    fps_average: f32,
}

impl SettingsPanel {
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            fps_average: 0.0,
        }
    }
}

impl Debug for SettingsPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsPanel")
            .field("id", &self.id)
            .field("fps_average", &self.fps_average)
            .finish()
    }
}

impl PostDeserialize for SettingsPanel {
    fn post_deserialize(&mut self) {}
}

impl SettingsPanel {
    fn fps_counter(&self) -> UIBox {
        let mut counter = text(
            format!("SettingsPanel{}_fps_counter", self.id),
            format!("FPS: {:.0}", self.fps_average),
        );

        counter.features |= UIBoxFeatureFlag::SkipTextCaching;

        counter
    }
}

impl PanelInstance for SettingsPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SETTINGS.with(|settings| -> Result<(), String> {
            let current_settings = settings.borrow();

            COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                let mut pending_queue = buffer.pending_commands.borrow_mut();

                // FPS counter

                self.fps_average = {
                    let new_fps =
                        GLOBAL_UI_CONTEXT.with(|ctx| ctx.timing_info.borrow().frames_per_second);

                    0.99 * self.fps_average + 0.01 * new_fps
                };

                tree.push(self.fps_counter())?;

                tree.push(spacer(18))?;

                // Setting: `clicked_count`

                tree.with_parent(
                    container(
                        format!("SettingsPanel{}_settings.clicked_count.container", self.id),
                        UILayoutDirection::LeftToRight,
                        None,
                    ),
                    |tree| {
                        if tree
                            .push(button(
                                format!(
                                    "SettingsPanel{}_settings.clicked_count.incrementButton",
                                    self.id
                                )
                                .to_string(),
                                "Click".to_string(),
                                None,
                            ))?
                            .mouse_interaction_in_bounds
                            .was_left_pressed
                        {
                            pending_queue.push_back(
                                format!(
                                    "set_setting clicked_count {}",
                                    current_settings.clicked_count + 1
                                )
                                .to_string(),
                            );
                        }

                        tree.push(spacer(18))?;

                        let clicked_count_text = {
                            let mut ui_box = text(
                                format!("SettingsPanel{}_settings.clicked_count.text", self.id)
                                    .to_string(),
                                format!("Clicked count: {}", current_settings.clicked_count)
                                    .to_string(),
                            );

                            ui_box.features |= UIBoxFeatureFlag::SkipTextCaching;

                            ui_box
                        };

                        tree.push(clicked_count_text)?;

                        Ok(())
                    },
                )?;

                Ok(())
            })
        })
    }
}
