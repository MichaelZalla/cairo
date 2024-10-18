use std::fmt::Debug;

use cairo::{
    app::{resolution::RESOLUTIONS_16X9, window::APP_WINDOWING_MODES},
    resource::handle::Handle,
    serde::PostDeserialize,
    ui::{
        fastpath::{
            checkbox::{checkbox_group, Checkbox},
            image::image,
            radio::{radio_group, RadioOption},
            slider::{slider, SliderOptions},
            spacer::spacer,
            text::text,
            text_input::text_input,
        },
        ui_box::tree::UIBoxTree,
        UISize, UISizeWithStrictness,
    },
};

use crate::{COMMAND_BUFFER, SETTINGS};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct SettingsPanel {
    id: String,
    fps_average: f32,
    test_text: String,
    test_image_handle: Handle,
}

impl Debug for SettingsPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsPanel")
            .field("id", &self.id)
            .field("fps_average", &self.fps_average)
            .field("test_text", &self.test_text)
            .field("test_image_handle", &self.test_image_handle)
            .finish()
    }
}

impl PostDeserialize for SettingsPanel {
    fn post_deserialize(&mut self) {}
}

impl SettingsPanel {
    pub fn new(id: &str, test_text: String, test_image_handle: Handle) -> Self {
        Self {
            id: id.to_string(),
            fps_average: 0.0,
            test_text,
            test_image_handle,
        }
    }
}

impl PanelInstance for SettingsPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SETTINGS.with(|settings| -> Result<(), String> {
            let current_settings = settings.borrow();

            COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                let mut pending_queue = buffer.pending_commands.borrow_mut();

                // Test text input

                tree.push(text(
                    format!("SettingsPanel{}.test_text_input.label", self.id).to_string(),
                    "A text input:".to_string(),
                ))?;

                if let Some(new_value) = text_input(
                    format!("SettingsPanel{}.test_text_input", self.id),
                    &self.test_text,
                    tree,
                )? {
                    self.test_text = new_value;
                }

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
                    let cmd_str = format!("set windowing_mode {}", index).to_string();

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
                    let cmd_str = format!("set resolution {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Brightness

                tree.push(text(
                    format!("SettingsPanel{}.brightness.label", self.id).to_string(),
                    "Brightness".to_string(),
                ))?;

                if let Some(new_brightness) = slider(
                    format!("SettingsPanel{}.brightness", self.id),
                    current_settings.brightness,
                    SliderOptions {
                        min: 0.0,
                        max: 1.0,
                        ..Default::default()
                    },
                    tree,
                )? {
                    let cmd_str = format!("set brightness {}", new_brightness).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Gamma

                tree.push(text(
                    format!("SettingsPanel{}.gamma.label", self.id).to_string(),
                    "Gamma".to_string(),
                ))?;

                if let Some(new_gamma) = slider(
                    format!("SettingsPanel{}.gamma", self.id),
                    current_settings.gamma,
                    SliderOptions {
                        min: 0.1,
                        max: 8.0,
                        ..Default::default()
                    },
                    tree,
                )? {
                    let cmd_str = format!("set gamma {}", new_gamma).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Miscellaneous flags

                tree.push(text(
                    format!("SettingsPanel{}.miscellaneous.label", self.id).to_string(),
                    "Miscellaneous".to_string(),
                ))?;

                let checkboxes = vec![
                    Checkbox::new("vsync", "Enable V-sync", current_settings.vsync),
                    Checkbox::new("hdr", "Enable HDR", current_settings.hdr),
                ];

                let toggled_indices = checkbox_group(
                    format!("SettingsPanel{}.miscellaneous.checkbox_group", self.id).to_string(),
                    &checkboxes,
                    tree,
                )?;

                for index in toggled_indices {
                    let checkbox = &checkboxes[index];

                    let cmd_str =
                        format!("set {} {}", checkbox.value, !checkbox.is_checked).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Test image

                tree.push(image(
                    format!("SettingsPanel{}.test_image1", self.id),
                    self.test_image_handle,
                    Some([
                        UISizeWithStrictness {
                            size: UISize::Pixels(192),
                            strictness: 0.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::Pixels(192),
                            strictness: 0.0,
                        },
                    ]),
                ))?;

                Ok(())
            })
        })
    }
}
