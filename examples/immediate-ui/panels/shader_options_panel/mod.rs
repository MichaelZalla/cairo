use std::fmt::Debug;

use cairo::{
    resource::handle::Handle,
    serde::PostDeserialize,
    ui::ui_box::{
        tree::UIBoxTree,
        utils::{spacer, text},
    },
};

use crate::{
    checkbox::{checkbox_group, Checkbox},
    radio::{radio_group, RadioOption},
    COMMAND_BUFFER, SETTINGS,
};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct ShaderOptionsPanel {
    id: String,
    renderer_handle: Handle,
}

impl Debug for ShaderOptionsPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderOptionsPanel")
            .field("id", &self.id)
            .field("renderer_handle", &self.renderer_handle)
            .finish()
    }
}

impl PostDeserialize for ShaderOptionsPanel {
    fn post_deserialize(&mut self) {}
}

impl ShaderOptionsPanel {
    pub fn new(id: &str, renderer_handle: Handle) -> Self {
        Self {
            id: id.to_string(),
            renderer_handle,
        }
    }
}

impl PanelInstance for ShaderOptionsPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SETTINGS.with(|settings| -> Result<(), String> {
            #[allow(unused)]
            let current_settings = settings.borrow();

            COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                let mut pending_queue = buffer.pending_commands.borrow_mut();

                // Texture filtering.

                tree.push(text(
                    format!("ShaderOptionsPanel{}.textureFiltering.label", self.id).to_string(),
                    "Texture filtering".to_string(),
                ))?;

                let texture_filtering_options: Vec<RadioOption> = [
                    "Nearest neighbors",
                    "Bilinear filtering",
                    "Trilinear filtering",
                    "Anisotropic filtering",
                ]
                .iter()
                .map(|label| RadioOption {
                    label: label.to_string(),
                })
                .collect();

                if let Some(index) = radio_group(
                    format!("ShaderOptionsPanel{}.textureFiltering.radioGroup", self.id)
                        .to_string(),
                    &texture_filtering_options,
                    1,
                    tree,
                )? {
                    let cmd_str = format!("set texture_filtering {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Texture maps.

                tree.push(text(
                    format!("ShaderOptionsPanel{}.textureMaps.label", self.id).to_string(),
                    "Texture maps".to_string(),
                ))?;

                let checkboxes = vec![
                    Checkbox::new("diffuseColorMapping", "Diffuse color maps", false),
                    Checkbox::new("ambientOcclusionMapping", "Ambient occlusion maps", true),
                    Checkbox::new("roughnessMapping", "Roughness maps", false),
                    Checkbox::new("metallicMapping", "Metallic maps", false),
                    Checkbox::new("normalMapping", "Normal maps", true),
                    Checkbox::new("displacementMapping", "Displacement maps", true),
                    Checkbox::new("specularMapping", "Specular maps", false),
                    Checkbox::new("emissiveMapping", "Emissive maps", true),
                ];

                let toggled_indices = checkbox_group(
                    format!("RenderOptionsPanel{}.textureMaps.checkbox_group", self.id).to_string(),
                    &checkboxes,
                    tree,
                )?;

                for index in toggled_indices {
                    let checkbox = &checkboxes[index];

                    let cmd_str =
                        format!("set {} {}", checkbox.value, !checkbox.is_checked).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                Ok(())
            })
        })
    }
}
