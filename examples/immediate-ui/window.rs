use uuid::Uuid;

use cairo::{
    app::resolution::Resolution,
    resource::{arena::Arena, handle::Handle},
    ui::{
        panel::{tree::PanelTree, Panel, PanelInstanceData, PanelRenderCallback},
        ui_box::UILayoutDirection,
        window::{list::WindowList, Window, WindowOptions},
    },
};

use crate::panels::{
    render_options_panel::RenderOptionsPanel, settings_panel::SettingsPanel,
    shader_options_panel::ShaderOptionsPanel,
};

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

pub(crate) fn make_window_list<'a>(
    settings_panel_arena: &mut Arena<SettingsPanel>,
    settings_panel_render_callback: PanelRenderCallback,
    render_options_panel_arena: &mut Arena<RenderOptionsPanel>,
    render_options_panel_render_callback: PanelRenderCallback,
    shader_options_panel_arena: &mut Arena<ShaderOptionsPanel>,
    shader_options_panel_render_callback: PanelRenderCallback,
    resolution: Resolution,
) -> Result<WindowList<'a>, String> {
    let mut list: WindowList = Default::default();

    // Builds a main "window" that encompasses our app's native OS window.

    let docked_window = {
        let window_id = "docked_window".to_string();
        let window_title = "Settings".to_string();
        let mut window_panel_tree = PanelTree::from_id(&window_id);

        let panel_id = format!("{}_SettingsPanel", window_id);

        let panel = make_settings_panel(
            panel_id.as_str(),
            settings_panel_arena,
            settings_panel_render_callback.clone(),
        );

        window_panel_tree.push(panel_id.as_str(), panel)?;

        Window::new(
            window_id,
            window_title,
            WindowOptions::docked(resolution),
            None,
            window_panel_tree,
        )
    };

    list.0.push_back(docked_window);

    // Builds a few non-native, "floating" windows that we can drag around.

    for i in 0..2 {
        let window_id = format!("floating_window_{}", i);

        let window_title;
        let mut window_panel_tree = PanelTree::from_id(&window_id);

        let panel_id;
        let panel_instance_data;

        if i == 0 {
            window_title = "Settings".to_string();

            panel_id = format!("{}_SettingsPanel", window_id);

            panel_instance_data = PanelInstanceData {
                panel_instance: settings_panel_arena
                    .insert(Uuid::new_v4(), SettingsPanel::from_id(panel_id.as_str())),
                render: Some(settings_panel_render_callback.clone()),
                custom_render_callback: None,
            };
        } else if i == 1 {
            window_title = "Render Options".to_string();

            panel_id = format!("{}_RenderOptionsPanel", window_id);
            panel_instance_data = PanelInstanceData {
                panel_instance: render_options_panel_arena.insert(
                    Uuid::new_v4(),
                    RenderOptionsPanel::new(
                        panel_id.as_str(),
                        Handle {
                            uuid: Uuid::new_v4(),
                            index: 0,
                        },
                    ),
                ),
                render: Some(render_options_panel_render_callback.clone()),
                custom_render_callback: None,
            };
        } else {
            window_title = "Shader Options".to_string();

            panel_id = format!("{}_ShaderOptionsPanel", window_id);
            panel_instance_data = PanelInstanceData {
                panel_instance: shader_options_panel_arena.insert(
                    Uuid::new_v4(),
                    ShaderOptionsPanel::new(
                        panel_id.as_str(),
                        Handle {
                            uuid: Uuid::new_v4(),
                            index: 0,
                        },
                    ),
                ),
                render: Some(shader_options_panel_render_callback.clone()),
                custom_render_callback: None,
            };
        }

        let panel = Panel::new(
            1.0,
            Some(panel_instance_data),
            UILayoutDirection::TopToBottom,
        );

        window_panel_tree.push(panel_id.as_str(), panel)?;

        list.0.push_back(Window::new(
            window_id,
            window_title,
            WindowOptions {
                with_titlebar: true,
                position: (50 + i * 250, 50),
                ..Default::default()
            },
            None,
            window_panel_tree,
        ));
    }

    Ok(list)
}
