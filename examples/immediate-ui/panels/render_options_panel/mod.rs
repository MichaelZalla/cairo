use std::fmt::Debug;

use cairo::{
    mem::linked_list::LinkedList,
    render::options::RenderPassFlag,
    resource::handle::Handle,
    serde::PostDeserialize,
    ui::{
        fastpath::{
            checkbox::{checkbox, checkbox_group, Checkbox},
            color::color_picker,
            container::collapsible_container,
            radio::{radio_group, RadioOption},
            slider::{slider, SliderOptions},
            spacer::spacer,
            text::text,
        },
        ui_box::tree::UIBoxTree,
    },
};

use crate::{command::PendingCommand, COMMAND_BUFFER, SETTINGS};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct RenderOptionsPanel {
    id: String,
    renderer_handle: Handle,
}

impl Debug for RenderOptionsPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderOptionsPanel")
            .field("id", &self.id)
            .field("renderer_handle", &self.renderer_handle)
            .finish()
    }
}

impl PostDeserialize for RenderOptionsPanel {
    fn post_deserialize(&mut self) {}
}

impl RenderOptionsPanel {
    pub fn new(id: &str, renderer_handle: Handle) -> Self {
        Self {
            id: id.to_string(),
            renderer_handle,
        }
    }
}

impl PanelInstance for RenderOptionsPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SETTINGS.with(|settings| -> Result<(), String> {
            let current_settings = settings.borrow();

            COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                let mut pending_queue = buffer.pending_commands.borrow_mut();

                // Render passes

                tree.push(text(
                    format!("{}.passes.label", self.id).to_string(),
                    "Render passes".to_string(),
                ))?;

                let render_pass_flags = vec![
                    Checkbox::new(
                        "render_options.do_rasterization",
                        "Rasterization",
                        current_settings
                            .render_options
                            .render_pass_flags
                            .contains(RenderPassFlag::Rasterization),
                    ),
                    Checkbox::new(
                        "render_options.do_lighting",
                        "Lighting",
                        current_settings
                            .render_options
                            .render_pass_flags
                            .contains(RenderPassFlag::Lighting),
                    ),
                    Checkbox::new(
                        "render_options.do_deferred_lighting",
                        "Deferred lighting",
                        current_settings
                            .render_options
                            .render_pass_flags
                            .contains(RenderPassFlag::DeferredLighting),
                    ),
                    Checkbox::new(
                        "render_options.do_bloom",
                        "Bloom",
                        current_settings
                            .render_options
                            .render_pass_flags
                            .contains(RenderPassFlag::Bloom),
                    ),
                ];

                for flag in render_pass_flags {
                    setting_checkbox(
                        &format!("{}.render_pass.{}", self.id, flag.value),
                        flag.value.as_str(),
                        flag.label.as_str(),
                        flag.is_checked,
                        tree,
                        &mut pending_queue,
                    )?;
                }

                tree.push(spacer(18))?;

                // Shaders

                tree.push(text(
                    format!("{}.shaders.label", self.id).to_string(),
                    "Shaders".to_string(),
                ))?;

                tree.push(text(
                    format!("{}.shaders.fragment.label", self.id).to_string(),
                    "Fragment shader".to_string(),
                ))?;

                let fragment_shader_options: Vec<RadioOption> = [
                    "Default",
                    "Debug - Albedo",
                    "Debug - Depth",
                    "Debug - Normal",
                    "Debug - UV",
                ]
                .iter()
                .map(|label| RadioOption {
                    label: label.to_string(),
                })
                .collect();

                if let Some(new_selected_fragment_shader_index) = radio_group(
                    format!("{}.shaders.fragment.radio_group", self.id).to_string(),
                    &fragment_shader_options,
                    current_settings.fragment_shader,
                    tree,
                )? {
                    let cmd_str = format!(
                        "set render.fragment_shader {}",
                        new_selected_fragment_shader_index
                    )
                    .to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Tone mapping

                tree.push(text(
                    format!("{}.render.tone_mapping.label", self.id).to_string(),
                    "Tone mapping".to_string(),
                ))?;

                let tone_mapping_options: Vec<RadioOption> =
                    ["Reinhard", "Exposure", "ACES Filmic", "AGX"]
                        .iter()
                        .map(|label| RadioOption {
                            label: label.to_string(),
                        })
                        .collect();

                if let Some(new_selected_tone_mapping_index) = radio_group(
                    format!("{}.render.tone_mapping.radio_group", self.id).to_string(),
                    &tone_mapping_options,
                    current_settings.tone_mapping,
                    tree,
                )? {
                    let cmd_str = format!(
                        "set render.tone_mapping {}",
                        new_selected_tone_mapping_index
                    )
                    .to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Shadows

                let label_box = text(
                    format!("{}.shadows.directionalShadows.label", self.id),
                    "Shadows".to_string(),
                );

                collapsible_container(
                    format!("{}.shadows", self.id).to_string(),
                    label_box,
                    tree,
                    |tree| -> Result<(), String> {
                        // Directional shadows

                        let directional_shadows_enabled = true;

                        setting_checkbox(
                            &format!("{}.shadows.directionalShadows", self.id),
                            "directionalShadows",
                            "Directional shadows",
                            directional_shadows_enabled,
                            tree,
                            &mut pending_queue,
                        )?;

                        tree.push(spacer(18))?;

                        if directional_shadows_enabled {
                            shadow_map_resolution_radio_group(
                                &format!("{}.shadows.directionalShadows", self.id),
                                "directionalShadowMapResolution",
                                0,
                                tree,
                                &mut pending_queue,
                            )?;

                            tree.push(spacer(18))?;
                        }

                        // Point shadows

                        let point_shadows_enabled = true;

                        setting_checkbox(
                            &format!("{}.shadows.pointShadows", self.id),
                            "pointShadows",
                            "Point shadows",
                            point_shadows_enabled,
                            tree,
                            &mut pending_queue,
                        )?;

                        if point_shadows_enabled {
                            tree.push(spacer(18))?;

                            shadow_map_resolution_radio_group(
                                &format!("{}.shadows.pointShadows", self.id),
                                "pointShadowMapResolution",
                                0,
                                tree,
                                &mut pending_queue,
                            )?;
                        }

                        tree.push(spacer(18))?;

                        Ok(())
                    },
                )?;

                // Post-processing

                tree.push(text(
                    format!("{}.postprocessing.label", self.id).to_string(),
                    "Postprocessing".to_string(),
                ))?;

                let checkboxes = vec![
                    Checkbox::new(
                        "postprocessing.effects.outline",
                        "Outline",
                        current_settings.effects.outline,
                    ),
                    Checkbox::new(
                        "postprocessing.effects.invert",
                        "Invert",
                        current_settings.effects.invert,
                    ),
                    Checkbox::new(
                        "postprocessing.effects.grayscale",
                        "Grayscale",
                        current_settings.effects.grayscale,
                    ),
                    Checkbox::new(
                        "postprocessing.effects.sharpen_kernel",
                        "Sharpen",
                        current_settings.effects.sharpen_kernel,
                    ),
                    Checkbox::new(
                        "postprocessing.effects.blur_kernel",
                        "Blur",
                        current_settings.effects.blur_kernel,
                    ),
                    Checkbox::new(
                        "postprocessing.effects.edge_detection_kernel",
                        "Edge detection",
                        current_settings.effects.edge_detection_kernel,
                    ),
                ];

                let toggled_indices = checkbox_group(
                    format!("{}.postprocessing.effects", self.id),
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

                // User debug

                tree.push(text(
                    format!("{}.debug.label", self.id).to_string(),
                    "Debug".to_string(),
                ))?;

                // Wireframe

                let draw_wireframe = current_settings.render_options.draw_wireframe;

                setting_checkbox(
                    &format!("{}.debug.drawWireframe", self.id),
                    "render_options.draw_wireframe",
                    "Draw wireframe",
                    draw_wireframe,
                    tree,
                    &mut pending_queue,
                )?;

                if draw_wireframe {
                    // Wireframe color.

                    let current_color = current_settings.render_options.wireframe_color;

                    tree.push(text(
                        format!("{}.debug.wireframe_color.label", self.id).to_string(),
                        "Wireframe color".to_string(),
                    ))?;

                    let color_picker_result = color_picker(
                        format!("{}.debug.wireframe_color", self.id),
                        current_color,
                        SliderOptions {
                            min: 0.0,
                            max: 1.0,
                            ..Default::default()
                        },
                        tree,
                    )?;

                    if let Some(new_color) = color_picker_result {
                        let cmd_str = format!(
                            "set render_options.wireframe_color ({:.2},{:.2},{:.2})",
                            new_color.x, new_color.y, new_color.z
                        )
                        .to_string();

                        pending_queue.push_back((cmd_str, false));
                    }

                    tree.push(spacer(18))?;
                }

                let draw_normals = current_settings.render_options.draw_normals;

                setting_checkbox(
                    &format!("{}.debug.drawNormals", self.id),
                    "render_options.draw_normals",
                    "Draw normals",
                    draw_normals,
                    tree,
                    &mut pending_queue,
                )?;

                if draw_normals {
                    // Draw normals scale

                    tree.push(text(
                        format!("SettingsPanel{}.debug.draw_normals_scale.label", self.id)
                            .to_string(),
                        "Scale".to_string(),
                    ))?;

                    if let Some(new_scale) = slider(
                        format!("SettingsPanel{}.debug.draw_normals_scale", self.id),
                        current_settings.render_options.draw_normals_scale,
                        SliderOptions {
                            min: 0.01,
                            max: 1.0,
                            ..Default::default()
                        },
                        tree,
                    )? {
                        let cmd_str =
                            format!("set render_options.draw_normals_scale {}", new_scale)
                                .to_string();

                        pending_queue.push_back((cmd_str, false));
                    }

                    tree.push(spacer(18))?;
                }

                Ok(())
            })
        })
    }
}

fn setting_checkbox(
    id: &str,
    setting: &str,
    label: &str,
    is_checked: bool,
    tree: &mut UIBoxTree,
    pending_queue: &mut LinkedList<PendingCommand>,
) -> Result<(), String> {
    if checkbox(
        &format!("{}.enabled", id),
        0,
        &Checkbox::new(setting, label, is_checked),
        tree,
    )? {
        let cmd_str = format!("set {} {}", setting, !is_checked).to_string();

        pending_queue.push_back((cmd_str, false));
    }

    Ok(())
}

fn shadow_map_resolution_radio_group(
    id: &str,
    setting: &str,
    selected_resolution_index: usize,
    tree: &mut UIBoxTree,
    pending_queue: &mut LinkedList<PendingCommand>,
) -> Result<(), String> {
    tree.push(text(
        format!("{}.shadowMapResolution.label", id).to_string(),
        "Shadow map resolution".to_string(),
    ))?;

    let resolution_options: Vec<RadioOption> = ["256x256", "512x512", "1024x1024"]
        .iter()
        .map(|label| RadioOption {
            label: label.to_string(),
        })
        .collect();

    if let Some(new_selected_resolution_index) = radio_group(
        format!("{}.shadowMapResolution.radio_group", id).to_string(),
        &resolution_options,
        selected_resolution_index,
        tree,
    )? {
        let cmd_str = format!("set {} {}", setting, new_selected_resolution_index).to_string();

        pending_queue.push_back((cmd_str, false));
    }

    Ok(())
}
