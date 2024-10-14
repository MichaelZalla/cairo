use uuid::Uuid;

use cairo::{
    resource::{arena::Arena, handle::Handle},
    ui::{
        panel::{tree::PanelTree, Panel, PanelInstanceData, PanelRenderCallback},
        ui_box::UILayoutDirection,
        window::{list::WindowList, Window, WindowOptions},
    },
};

use crate::panels::{
    rasterization_options_panel::RasterizationOptionsPanel,
    render_options_panel::RenderOptionsPanel, settings_panel::SettingsPanel,
    shader_options_panel::ShaderOptionsPanel, PanelArenas, PanelRenderCallbacks,
};

#[allow(unused)]
fn make_settings_panel(
    id: &str,
    arena: &mut Arena<SettingsPanel>,
    render_callback: PanelRenderCallback,
) -> Panel {
    let settings_panel_instance_data = PanelInstanceData {
        panel_instance: arena.insert(Uuid::new_v4(), SettingsPanel::from_id(id)),
        render: Some(render_callback.clone()),
        custom_render_callback: None,
    };

    Panel::new(
        1.0,
        Some(settings_panel_instance_data),
        UILayoutDirection::TopToBottom,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_window_list<'a>(
    panel_arenas: PanelArenas,
    panel_render_callbacks: PanelRenderCallbacks,
) -> Result<WindowList<'a>, String> {
    let mut list: WindowList = Default::default();

    // Builds a few non-native, "floating" windows that we can drag around.

    for i in 0..4 {
        let window_id = format!("floating_window_{}", i);

        let window_title;
        let panel_id;
        let panel_instance_data;

        match i {
            0 => {
                let mut arena = panel_arenas.settings.borrow_mut();

                window_title = "Settings".to_string();

                panel_id = format!("{}_SettingsPanel", window_id);

                panel_instance_data = PanelInstanceData {
                    panel_instance: arena
                        .insert(Uuid::new_v4(), SettingsPanel::from_id(panel_id.as_str())),
                    render: Some(panel_render_callbacks.settings.clone()),
                    custom_render_callback: None,
                };
            }
            1 => {
                let mut arena = panel_arenas.render_options.borrow_mut();

                window_title = "Render Options".to_string();

                panel_id = format!("{}_RenderOptionsPanel", window_id);
                panel_instance_data = PanelInstanceData {
                    panel_instance: arena.insert(
                        Uuid::new_v4(),
                        RenderOptionsPanel::new(
                            panel_id.as_str(),
                            Handle {
                                uuid: Uuid::new_v4(),
                                index: 0,
                            },
                        ),
                    ),
                    render: Some(panel_render_callbacks.render_options.clone()),
                    custom_render_callback: None,
                };
            }
            2 => {
                let mut arena = panel_arenas.shader_options.borrow_mut();

                window_title = "Texture Options".to_string();

                panel_id = format!("{}_ShaderOptionsPanel", window_id);
                panel_instance_data = PanelInstanceData {
                    panel_instance: arena.insert(
                        Uuid::new_v4(),
                        ShaderOptionsPanel::new(
                            panel_id.as_str(),
                            Handle {
                                uuid: Uuid::new_v4(),
                                index: 0,
                            },
                        ),
                    ),
                    render: Some(panel_render_callbacks.shader_options.clone()),
                    custom_render_callback: None,
                };
            }
            _ => {
                let mut arena = panel_arenas.rasterization_options.borrow_mut();

                window_title = "Rasterization Options".to_string();

                panel_id = format!("{}_RasterizationOptionsPanel", window_id);
                panel_instance_data = PanelInstanceData {
                    panel_instance: arena.insert(
                        Uuid::new_v4(),
                        RasterizationOptionsPanel::new(
                            panel_id.as_str(),
                            Handle {
                                uuid: Uuid::new_v4(),
                                index: 0,
                            },
                        ),
                    ),
                    render: Some(panel_render_callbacks.rasterization_options.clone()),
                    custom_render_callback: None,
                };
            }
        }

        let mut window_panel_tree = PanelTree::from_id(&window_id);

        window_panel_tree.push(
            panel_id.as_str(),
            Panel::new(
                1.0,
                Some(panel_instance_data),
                UILayoutDirection::TopToBottom,
            ),
        )?;

        let window = Window::new(
            window_id,
            window_title,
            WindowOptions {
                with_titlebar: true,
                position: (50 + i * 250, 50),
                ..Default::default()
            },
            None,
            window_panel_tree,
        );

        list.0.push_back(window);
    }

    Ok(list)
}
