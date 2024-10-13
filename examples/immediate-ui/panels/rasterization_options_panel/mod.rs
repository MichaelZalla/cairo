use std::fmt::Debug;

use cairo::{
    resource::handle::Handle,
    serde::PostDeserialize,
    software_renderer::zbuffer::DEPTH_TEST_METHODS,
    ui::ui_box::{
        tree::UIBoxTree,
        utils::{spacer, text},
    },
};

use crate::{
    radio::{radio_group, RadioOption},
    COMMAND_BUFFER, SETTINGS,
};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct RasterizationOptionsPanel {
    id: String,
    renderer_handle: Handle,
}

impl Debug for RasterizationOptionsPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RasterizationOptionsPanel")
            .field("id", &self.id)
            .field("renderer_handle", &self.renderer_handle)
            .finish()
    }
}

impl PostDeserialize for RasterizationOptionsPanel {
    fn post_deserialize(&mut self) {}
}

impl RasterizationOptionsPanel {
    pub fn new(id: &str, renderer_handle: Handle) -> Self {
        Self {
            id: id.to_string(),
            renderer_handle,
        }
    }
}

impl PanelInstance for RasterizationOptionsPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SETTINGS.with(|settings| -> Result<(), String> {
            #[allow(unused)]
            let current_settings = settings.borrow();

            COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                let mut pending_queue = buffer.pending_commands.borrow_mut();

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
                    let cmd_str = format!("set faceWinding {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

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
                    let cmd_str = format!("set faceCulling {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Depth test method

                let depth_test_method_index = current_settings.depth_test_method as usize;

                tree.push(text(
                    format!("RenderOptionsPanel{}.depthTestingMethod.label", self.id).to_string(),
                    "Depth test".to_string(),
                ))?;

                let depth_testing_method_options: Vec<RadioOption> = DEPTH_TEST_METHODS
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
                    depth_test_method_index,
                    tree,
                )? {
                    let cmd_str = format!("set depth_test_method {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                Ok(())
            })
        })
    }
}
