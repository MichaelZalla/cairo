use std::fmt::Debug;

use cairo::{
    mem::linked_list::LinkedList,
    resource::handle::Handle,
    serde::PostDeserialize,
    software_renderer::zbuffer::DepthTestMethod,
    ui::ui_box::{
        tree::UIBoxTree,
        utils::{spacer, text},
    },
};

use crate::{
    checkbox::{checkbox, Checkbox},
    command::PendingCommand,
    radio::{radio_group, RadioOption},
    COMMAND_BUFFER, SETTINGS,
};

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

                let wireframe_enabled = true;

                setting_checkbox(
                    &format!("{}.render_pass.wireframe", self.id),
                    "wireframe",
                    "Wireframe",
                    wireframe_enabled,
                    tree,
                    &mut pending_queue,
                )?;

                if wireframe_enabled {
                    // Wireframe color.

                    tree.push(text(
                        format!("{}.render_pass.wireframe_color.label", self.id).to_string(),
                        "Wireframe color".to_string(),
                    ))?;

                    tree.push(text(
                        format!("{}.render_pass.wireframe_color", self.id).to_string(),
                        "[_____________]".to_string(),
                    ))?;
                }

                tree.push(spacer(18))?;

                let render_pass_flags = vec![
                    // Checkbox::new("wireframe", "Wireframe", false),
                    Checkbox::new("rasterization", "Rasterization", true),
                    Checkbox::new("lighting", "Lighting", true),
                    Checkbox::new("deferredLighting", "Deferred lighting", false),
                    Checkbox::new("visualizeNormals", "Visualize normals", false),
                    Checkbox::new("bloom", "Bloom", current_settings.bloom),
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

                // Face culling

                tree.push(text(
                    format!("RenderOptionsPanel{}.faceCulling.label", self.id).to_string(),
                    "Face culling".to_string(),
                ))?;

                let reject_faces_options: Vec<RadioOption> =
                    ["Reject backfaces", "Reject frontfaces", "Disabled"]
                        .iter()
                        .map(|label| RadioOption {
                            label: label.to_string(),
                        })
                        .collect();

                if let Some(index) = radio_group(
                    format!("RenderOptionsPanel{}.faceCulling.radio_group", self.id).to_string(),
                    &reject_faces_options,
                    0,
                    tree,
                )? {
                    let cmd_str = format!("set_setting faceCulling {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Depth test method

                tree.push(text(
                    format!("RenderOptionsPanel{}.depthTestingMethod.label", self.id).to_string(),
                    "Depth test".to_string(),
                ))?;

                let depth_testing_method_options: Vec<RadioOption> = [
                    DepthTestMethod::Always,
                    DepthTestMethod::Never,
                    DepthTestMethod::Less,
                    DepthTestMethod::Equal,
                    DepthTestMethod::LessThanOrEqual,
                    DepthTestMethod::Greater,
                    DepthTestMethod::NotEqual,
                    DepthTestMethod::GreaterThanOrEqual,
                ]
                .iter()
                .map(|label| RadioOption {
                    label: label.to_string(),
                })
                .collect();

                if let Some(index) = radio_group(
                    format!(
                        "RenderOptionsPanel{}.depthTestingMethod.radio_group",
                        self.id
                    )
                    .to_string(),
                    &depth_testing_method_options,
                    2,
                    tree,
                )? {
                    let cmd_str = format!("set_setting depthTestingMethod {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Face winding

                tree.push(text(
                    format!("RenderOptionsPanel{}.faceWinding.label", self.id).to_string(),
                    "Face winding".to_string(),
                ))?;

                let reject_faces_options: Vec<RadioOption> = ["Counter-clockwise", "Clockwise"]
                    .iter()
                    .map(|label| RadioOption {
                        label: label.to_string(),
                    })
                    .collect();

                if let Some(index) = radio_group(
                    format!("RenderOptionsPanel{}.faceWinding.radio_group", self.id).to_string(),
                    &reject_faces_options,
                    0,
                    tree,
                )? {
                    let cmd_str = format!("set_setting faceWinding {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Shadows

                tree.push(text(
                    format!("{}.shadows.label", self.id).to_string(),
                    "Shadows".to_string(),
                ))?;

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

                if directional_shadows_enabled {
                    shadow_map_resolution_radio_group(
                        &format!("{}.shadows.directionalShadows", self.id),
                        "directionalShadowMapResolution",
                        0,
                        tree,
                        &mut pending_queue,
                    )?;
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
                    shadow_map_resolution_radio_group(
                        &format!("{}.shadows.pointShadows", self.id),
                        "pointShadowMapResolution",
                        0,
                        tree,
                        &mut pending_queue,
                    )?;
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
        let cmd_str = format!("set_setting {} {}", setting, !is_checked).to_string();

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
        let cmd_str =
            format!("set_setting {} {}", setting, new_selected_resolution_index).to_string();

        pending_queue.push_back((cmd_str, false));
    }

    Ok(())
}
