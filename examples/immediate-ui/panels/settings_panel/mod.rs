use std::fmt::Debug;

use cairo::{
    app::{resolution::RESOLUTIONS_16X9, window::APP_WINDOWING_MODES},
    mem::linked_list::LinkedList,
    serde::PostDeserialize,
    ui::{
        context::GLOBAL_UI_CONTEXT,
        ui_box::{
            tree::UIBoxTree,
            utils::{spacer, text},
            UIBox, UIBoxFeatureFlag,
        },
    },
};

use crate::{
    checkbox::{checkbox_group, Checkbox},
    command::ExecutedCommand,
    radio::{radio_group, RadioOption},
    COMMAND_BUFFER, SETTINGS,
};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct SettingsPanel {
    id: String,
    fps_average: f32,
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
    pub fn from_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            fps_average: 0.0,
        }
    }

    fn fps_counter(&self) -> UIBox {
        let mut counter = text(
            format!("SettingsPanel{}_fps_counter", self.id),
            format!("FPS: {:.0}", self.fps_average),
        );

        counter.features |= UIBoxFeatureFlag::SkipTextCaching;

        counter
    }

    fn command_history(
        &self,
        executed_queue: &LinkedList<ExecutedCommand>,
        tree: &mut UIBoxTree,
    ) -> Result<(), String> {
        static RECENT_COMMANDS_COUNT: usize = 3;

        tree.push(text(
            format!("SettingsPanel{}.command_history.label", self.id).to_string(),
            format!(
                "Command history{}",
                if executed_queue.is_empty() {
                    "".to_string()
                } else {
                    format!(" ({})", executed_queue.len())
                }
            )
            .to_string(),
        ))?;

        if executed_queue.is_empty() {
            tree.push(text(
                format!("SettingsPanel{}.command_history.most_recent_empty", self.id,).to_string(),
                "No history.".to_string(),
            ))?;
        } else {
            for (index, cmd) in executed_queue.iter().rev().enumerate() {
                let cmd_serialized = format!("{} {}", cmd.kind, cmd.args.join(" ")).to_string();

                tree.push(text(
                    format!(
                        "SettingsPanel{}.command_history.most_recent_{}",
                        self.id, index
                    )
                    .to_string(),
                    format!("{}: {}", index, cmd_serialized).to_string(),
                ))?;

                if index > RECENT_COMMANDS_COUNT {
                    break;
                }
            }
        }

        Ok(())
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

                // Windowing mode.

                tree.push(text(
                    format!("SettingsPanel{}.windowing_mode.label", self.id).to_string(),
                    "Windowing mode".to_string(),
                ))?;

                let windowing_mode_options: Vec<RadioOption> = APP_WINDOWING_MODES
                    .iter()
                    .map(|mode| RadioOption {
                        label: mode.to_string(),
                    })
                    .collect();

                if let Some(index) = radio_group(
                    format!("SettingsPanel{}.windowing_mode.radio_group", self.id).to_string(),
                    &windowing_mode_options,
                    current_settings.windowing_mode as usize,
                    tree,
                )? {
                    let cmd_str = format!("set_setting windowing_mode {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Window resolution

                tree.push(text(
                    format!("SettingsPanel{}.resolution.label", self.id).to_string(),
                    "Resolution".to_string(),
                ))?;

                let resolution_options: Vec<RadioOption> = RESOLUTIONS_16X9
                    .iter()
                    .map(|resolution| RadioOption {
                        label: format!("{}x{}", resolution.width, resolution.height),
                    })
                    .collect();

                if let Some(index) = radio_group(
                    format!("SettingsPanel{}.resolution.radio_group", self.id).to_string(),
                    &resolution_options,
                    current_settings.resolution,
                    tree,
                )? {
                    let cmd_str = format!("set_setting resolution {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Postprocessing flags

                tree.push(text(
                    format!("SettingsPanel{}.postprocessing.label", self.id).to_string(),
                    "Postprocessing".to_string(),
                ))?;

                let checkboxes = vec![
                    Checkbox::new("vsync", "Enable V-sync", current_settings.vsync),
                    Checkbox::new("hdr", "Enable HDR", current_settings.hdr),
                ];

                let toggled_indices = checkbox_group(
                    format!("SettingsPanel{}.postprocessing.checkbox_group", self.id).to_string(),
                    &checkboxes,
                    tree,
                )?;

                for index in toggled_indices {
                    let checkbox = &checkboxes[index];

                    let cmd_str =
                        format!("set_setting {} {}", checkbox.value, !checkbox.is_checked)
                            .to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Command history

                let executed_queue = buffer.executed_commands.borrow();

                self.command_history(&executed_queue, tree)
            })
        })
    }
}
