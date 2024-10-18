use uuid::Uuid;

use cairo::{
    resource::handle::Handle,
    scene::context::SceneContext,
    ui::{
        panel::{tree::PanelTree, Panel, PanelInstanceData},
        ui_box::UILayoutDirection,
        window::{list::WindowList, Window, WindowOptions},
    },
};

use crate::panels::{
    camera_attributes_panel::CameraAttributesPanel,
    rasterization_options_panel::RasterizationOptionsPanel,
    render_options_panel::RenderOptionsPanel, settings_panel::SettingsPanel,
    shader_options_panel::ShaderOptionsPanel, PanelArenas, PanelRenderCallbacks,
};

pub(crate) fn make_window_list<'a>(
    scene_context: &SceneContext,
    panel_arenas: &PanelArenas,
    panel_render_callbacks: PanelRenderCallbacks,
    uv_test_gradient_texture_handle: &Handle,
) -> Result<WindowList<'a>, String> {
    let mut list: WindowList = Default::default();

    // Builds a few non-native, "floating" windows that we can drag around.

    for i in 0..5 {
        let window_id = format!("floating_window_{}", i);

        let window_title;
        let panel_id;
        let panel_instance_data;

        match i {
            0 => {
                let mut panel_arena = panel_arenas.settings.borrow_mut();

                window_title = "Settings".to_string();

                panel_id = format!("{}_SettingsPanel", window_id);

                panel_instance_data = PanelInstanceData {
                    panel_instance: panel_arena.insert(SettingsPanel::new(
                        panel_id.as_str(),
                        "Hello, world!".to_string(),
                        *uv_test_gradient_texture_handle,
                    )),
                    render: Some(panel_render_callbacks.settings.clone()),
                    custom_render_callback: None,
                };
            }
            1 => {
                let mut panel_arena = panel_arenas.render_options.borrow_mut();

                window_title = "Render Options".to_string();

                panel_id = format!("{}_RenderOptionsPanel", window_id);
                panel_instance_data = PanelInstanceData {
                    panel_instance: panel_arena.insert(RenderOptionsPanel::new(
                        panel_id.as_str(),
                        Handle {
                            uuid: Uuid::new_v4(),
                            index: 0,
                        },
                    )),
                    render: Some(panel_render_callbacks.render_options.clone()),
                    custom_render_callback: None,
                };
            }
            2 => {
                let mut panel_arena = panel_arenas.shader_options.borrow_mut();

                window_title = "Texture Options".to_string();

                panel_id = format!("{}_ShaderOptionsPanel", window_id);
                panel_instance_data = PanelInstanceData {
                    panel_instance: panel_arena.insert(ShaderOptionsPanel::new(
                        panel_id.as_str(),
                        Handle {
                            uuid: Uuid::new_v4(),
                            index: 0,
                        },
                    )),
                    render: Some(panel_render_callbacks.shader_options.clone()),
                    custom_render_callback: None,
                };
            }
            3 => {
                let mut panel_arena = panel_arenas.rasterization_options.borrow_mut();

                window_title = "Rasterization Options".to_string();

                panel_id = format!("{}_RasterizationOptionsPanel", window_id);
                panel_instance_data = PanelInstanceData {
                    panel_instance: panel_arena.insert(RasterizationOptionsPanel::new(
                        panel_id.as_str(),
                        Handle {
                            uuid: Uuid::new_v4(),
                            index: 0,
                        },
                    )),
                    render: Some(panel_render_callbacks.rasterization_options.clone()),
                    custom_render_callback: None,
                };
            }
            _ => {
                let scene_resources = scene_context.resources.borrow();

                let camera_arena = scene_resources.camera.borrow();

                if let Some(entry) = &camera_arena.entries[0] {
                    let camera_handle = Handle {
                        index: 0,
                        uuid: entry.uuid,
                    };

                    let mut panel_arena = panel_arenas.camera_attributes.borrow_mut();

                    window_title = "Camera".to_string();

                    panel_id = format!("{}_CameraAttributesPanel", window_id);
                    panel_instance_data = PanelInstanceData {
                        panel_instance: panel_arena
                            .insert(CameraAttributesPanel::new(panel_id.as_str(), camera_handle)),
                        render: Some(panel_render_callbacks.camera_attributes.clone()),
                        custom_render_callback: None,
                    };
                } else {
                    panic!()
                }
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
