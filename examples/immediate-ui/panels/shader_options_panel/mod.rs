use std::fmt::Debug;

use cairo::{
    resource::handle::Handle,
    serde::PostDeserialize,
    ui::{
        fastpath::{
            checkbox::{Checkbox, checkbox_group},
            container::collapsible_container,
            radio::{RadioOption, radio_group},
            spacer::spacer,
            text::text,
        },
        ui_box::tree::UIBoxTree,
    },
};

use crate::{COMMAND_BUFFER, SETTINGS};

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
                ]
                .iter()
                .map(|label| RadioOption {
                    label: label.to_string(),
                })
                .collect();

                let selected_index = match (
                    current_settings.shader_options.bilinear_active,
                    current_settings.shader_options.trilinear_active,
                ) {
                    (true, true) => panic!(),
                    (true, false) => 1,
                    (false, true) => 2,
                    (false, false) => 0,
                };

                if let Some(index) = radio_group(
                    format!("ShaderOptionsPanel{}.textureFiltering.radioGroup", self.id)
                        .to_string(),
                    &texture_filtering_options,
                    selected_index,
                    tree,
                )? {
                    let cmd_str =
                        format!("set shader_options.texture_filtering {}", index).to_string();

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;

                // Texture maps.

                let label_box = text(
                    format!("ShaderOptionsPanel{}.textureMaps.label", self.id,),
                    "Texture mapping".to_string(),
                );

                collapsible_container(
                    format!("ShaderOptionsPanel{}.textureMaps", self.id).to_string(),
                    label_box,
                    tree,
                    |tree| -> Result<(), String> {
                        let checkboxes = vec![
                            Checkbox::new(
                                "shader_options.albedo_color_maps",
                                "Albedo color maps",
                                current_settings.shader_options.albedo_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.ambient_occlusion_maps",
                                "Ambient occlusion maps",
                                current_settings
                                    .shader_options
                                    .ambient_occlusion_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.roughness_maps",
                                "Roughness maps",
                                current_settings.shader_options.roughness_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.metallic_maps",
                                "Metallic maps",
                                current_settings.shader_options.metallic_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.normal_maps",
                                "Normal maps",
                                current_settings.shader_options.normal_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.displacement_maps",
                                "Displacement maps",
                                current_settings.shader_options.displacement_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.specular_maps",
                                "Specular maps",
                                current_settings
                                    .shader_options
                                    .specular_exponent_mapping_active,
                            ),
                            Checkbox::new(
                                "shader_options.emissive_maps",
                                "Emissive maps",
                                current_settings
                                    .shader_options
                                    .emissive_color_mapping_active,
                            ),
                        ];

                        let toggled_indices = checkbox_group(
                            format!("ShaderOptionsPanel{}.textureMaps.checkbox_group", self.id)
                                .to_string(),
                            &checkboxes,
                            tree,
                        )
                        .unwrap();

                        for index in toggled_indices {
                            let checkbox = &checkboxes[index];

                            let cmd_str =
                                format!("set {} {}", checkbox.value, !checkbox.is_checked)
                                    .to_string();

                            pending_queue.push_back((cmd_str, false));
                        }

                        Ok(())
                    },
                )?;

                Ok(())
            })
        })
    }
}
