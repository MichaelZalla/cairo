use cairo::{
    app::resolution::Resolution,
    ui::{
        panel::{tree::PanelTree, Panel},
        ui_box::UILayoutDirection,
        window::{list::WindowList, Window, WindowOptions},
    },
};

pub(crate) fn make_window_list<'a>(resolution: Resolution) -> Result<WindowList<'a>, String> {
    let mut list: WindowList = Default::default();

    // Builds a main "window" that encompasses our app's native OS window.

    let main_window = {
        let window_id = "main_window".to_string();

        let mut window_panel_tree = PanelTree::from_id(&window_id);

        window_panel_tree.push(
            "Something",
            Panel::new(1.0, None, UILayoutDirection::TopToBottom),
        )?;

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

        window_panel_tree.push(
            "Something",
            Panel::new(1.0, None, UILayoutDirection::TopToBottom),
        )?;

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
