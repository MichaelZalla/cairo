use uuid::Uuid;

use cairo::{
    app::resolution::Resolution,
    resource::arena::Arena,
    ui::{
        panel::{tree::PanelTree, Panel, PanelInstanceData, PanelRenderCallback},
        ui_box::UILayoutDirection,
        window::{list::WindowList, Window, WindowOptions},
    },
};

use crate::SettingsPanel;

pub(crate) fn make_window_list<'a>(
    button_panel_arena: &mut Arena<SettingsPanel>,
    button_panel_render_callback: PanelRenderCallback,
    resolution: Resolution,
) -> Result<WindowList<'a>, String> {
    let mut list: WindowList = Default::default();

    // Builds a main "window" that encompasses our app's native OS window.

    let main_window = {
        let window_id = "main_window".to_string();

        let mut window_panel_tree = PanelTree::from_id(&window_id);

        let button_panel_data = PanelInstanceData {
            panel_instance: button_panel_arena
                .insert(Uuid::new_v4(), SettingsPanel::from_id("main")),
            render: Some(button_panel_render_callback.clone()),
            custom_render_callback: None,
        };

        let button_panel = Panel::new(1.0, Some(button_panel_data), UILayoutDirection::TopToBottom);

        window_panel_tree.push("SettingsPanel_main", button_panel)?;

        Window::new(
            window_id,
            WindowOptions::docked(resolution),
            None,
            window_panel_tree,
        )
    };

    list.0.push_back(main_window);

    // Builds a few non-native, "floating" windows that we can drag around.

    for i in 0..3 {
        let window_id = format!("floating_window_{}", i);

        let mut window_panel_tree = PanelTree::from_id(&window_id);

        let button_panel_data = PanelInstanceData {
            panel_instance: button_panel_arena.insert(
                Uuid::new_v4(),
                SettingsPanel::from_id(format!("{}", i).as_str()),
            ),
            render: Some(button_panel_render_callback.clone()),
            custom_render_callback: None,
        };

        let button_panel = Panel::new(1.0, Some(button_panel_data), UILayoutDirection::TopToBottom);

        window_panel_tree.push(&format!("ButtonPanel_{}", i), button_panel)?;

        list.0.push_back(Window::new(
            window_id,
            WindowOptions {
                with_titlebar: true,
                position: (50 + i * 50, 50 + i * 50),
                size: (320, 250),
                ..Default::default()
            },
            None,
            window_panel_tree,
        ));
    }

    Ok(list)
}
