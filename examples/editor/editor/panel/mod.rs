use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use cairo::{
    buffer::Buffer2D,
    resource::arena::Arena,
    scene::{camera::Camera, resources::SceneResources},
    software_renderer::SoftwareRenderer,
    ui::{
        extent::ScreenExtent,
        panel::{tree::PanelTree, Panel, PanelInstanceData, PanelRenderCallback},
        ui_box::{tree::UIBoxTree, UIBoxCustomRenderCallback, UILayoutDirection},
    },
    vec::vec3::{self, Vec3},
};

use asset_browser::AssetBrowserPanel;
use console::ConsolePanel;
use file_system::FileSystemPanel;
use inspector::InspectorPanel;
use outline::OutlinePanel;
use viewport_3d::Viewport3DPanel;

pub mod asset_browser;
pub mod console;
pub mod file_system;
pub mod inspector;
pub mod outline;
pub mod viewport_3d;

pub trait PanelInstance {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String>;

    fn custom_render_callback(
        &mut self,
        _extent: &ScreenExtent,
        _target: &mut Buffer2D,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EditorPanelArenas {
    pub outline: RefCell<Arena<OutlinePanel>>,
    pub viewport_3d: RefCell<Arena<Viewport3DPanel>>,
    pub asset_browser: RefCell<Arena<AssetBrowserPanel>>,
    pub console: RefCell<Arena<ConsolePanel>>,
    pub inspector: RefCell<Arena<InspectorPanel>>,
    pub file_system: RefCell<Arena<FileSystemPanel>>,
}

thread_local! {
    pub static EDITOR_PANEL_ARENAS: EditorPanelArenas = Default::default();
}

pub struct EditorPanelRenderCallbacks {
    pub outline: PanelRenderCallback,
    pub viewport_3d: (PanelRenderCallback, UIBoxCustomRenderCallback),
    pub asset_browser: PanelRenderCallback,
    pub console: PanelRenderCallback,
    pub inspector: PanelRenderCallback,
    pub file_system: PanelRenderCallback,
}

pub fn build_floating_window_panel_tree<'a>(
    window_id: &String,
    panel_instance_data: PanelInstanceData,
) -> Result<PanelTree<'a>, String> {
    Ok(PanelTree::with_root(Panel {
        path: format!("{}_root", window_id),
        resizable: true,
        alpha_split: 1.0,
        instance_data: Some(panel_instance_data),
        layout_direction: UILayoutDirection::TopToBottom,
    }))
}

pub fn build_main_window_panel_tree<'a>(
    window_id: &String,
    resource_arenas: &SceneResources,
    panel_arenas: &EditorPanelArenas,
    panel_render_callbacks: &EditorPanelRenderCallbacks,
    renderer: &Rc<RefCell<SoftwareRenderer>>,
) -> Result<PanelTree<'a>, String> {
    let mut tree = PanelTree::with_root(Panel {
        path: format!("{}_root", window_id),
        resizable: true,
        alpha_split: 1.0,
        instance_data: None,
        layout_direction: UILayoutDirection::LeftToRight,
    });

    let mut camera_arena = resource_arenas.camera.borrow_mut();

    let mut cameras = [
        Camera::from_perspective(
            Vec3 {
                x: 4.0,
                y: 4.0,
                z: -4.0,
            },
            Default::default(),
            75.0,
            16.0 / 9.0,
        ),
        Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            vec3::FORWARD,
            75.0,
            16.0 / 9.0,
        ),
        Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            -vec3::UP
                + Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0001,
                },
            75.0,
            16.0 / 9.0,
        ),
        Camera::from_perspective(
            Vec3 {
                x: -5.0,
                y: 0.0,
                z: 0.0,
            },
            vec3::RIGHT,
            75.0,
            16.0 / 9.0,
        ),
    ];

    for (index, camera) in cameras.iter_mut().enumerate() {
        camera.movement_speed = 5.0;

        if index != 0 {
            camera.set_projection_z_far(25.0);
        }
    }

    // Root > Left.

    tree.push_parent(
        "Left",
        Panel::new(0.2, None, UILayoutDirection::TopToBottom),
    )?;

    // Root > Left > Top (Outline).

    tree.push(
        "Top",
        Panel::new(
            0.5,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.outline.clone()),
                custom_render_callback: None,
                panel_instance: panel_arenas.outline.borrow_mut().insert(Default::default()),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Root > Left > Bottom (AssetBrowser).

    tree.push(
        "Bottom",
        Panel::new(
            0.5,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.asset_browser.clone()),
                custom_render_callback: None,
                panel_instance: panel_arenas
                    .asset_browser
                    .borrow_mut()
                    .insert(Default::default()),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to Root > Bottom.

    tree.pop_parent()?;

    // Root > Middle (3D Viewports, Console).

    tree.push_parent(
        "Middle",
        Panel::new(0.6, None, UILayoutDirection::TopToBottom),
    )?;

    // Root > Middle > Top (2x2 Viewports).

    tree.push_parent("Top", Panel::new(0.7, None, UILayoutDirection::TopToBottom))?;

    // Root > Middle > Top > Top (1x2 Viewports).

    tree.push_parent("Top", Panel::new(0.5, None, UILayoutDirection::LeftToRight))?;

    tree.push(
        "Left",
        Panel::new(
            0.5,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.viewport_3d.0.clone()),
                custom_render_callback: Some(panel_render_callbacks.viewport_3d.1),
                panel_instance: panel_arenas
                    .viewport_3d
                    .borrow_mut()
                    .insert(Viewport3DPanel::new(
                        renderer.clone(),
                        camera_arena.insert(cameras[0]),
                    )),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Right",
        Panel::new(
            0.5,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.viewport_3d.0.clone()),
                custom_render_callback: Some(panel_render_callbacks.viewport_3d.1),
                panel_instance: panel_arenas
                    .viewport_3d
                    .borrow_mut()
                    .insert(Viewport3DPanel::new(
                        renderer.clone(),
                        camera_arena.insert(cameras[1]),
                    )),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to Root > Bottom > Middle > Top.

    tree.pop_parent()?;

    // Root > Middle > Bottom (1x2 Viewports).

    tree.push_parent(
        "Bottom",
        Panel::new(0.5, None, UILayoutDirection::LeftToRight),
    )?;

    tree.push(
        "Left",
        Panel::new(
            0.5,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.viewport_3d.0.clone()),
                custom_render_callback: Some(panel_render_callbacks.viewport_3d.1),
                panel_instance: panel_arenas
                    .viewport_3d
                    .borrow_mut()
                    .insert(Viewport3DPanel::new(
                        renderer.clone(),
                        camera_arena.insert(cameras[2]),
                    )),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Right",
        Panel::new(
            0.5,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.viewport_3d.0.clone()),
                custom_render_callback: Some(panel_render_callbacks.viewport_3d.1),
                panel_instance: panel_arenas
                    .viewport_3d
                    .borrow_mut()
                    .insert(Viewport3DPanel::new(
                        renderer.clone(),
                        camera_arena.insert(cameras[3]),
                    )),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to Root > Bottom > Middle > Top.

    tree.pop_parent()?;

    // Back to Root > Bottom > Middle.

    tree.pop_parent()?;

    // Root > Middle > Bottom (Console).

    tree.push(
        "Bottom",
        Panel::new(
            0.3,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.console.clone()),
                custom_render_callback: None,
                panel_instance: panel_arenas.console.borrow_mut().insert(Default::default()),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to Root.

    tree.pop_parent()?;

    // Root > Right (Inspector).

    tree.push_parent(
        "Right",
        Panel::new(
            0.2,
            Some(PanelInstanceData {
                render: Some(panel_render_callbacks.inspector.clone()),
                custom_render_callback: None,
                panel_instance: panel_arenas
                    .inspector
                    .borrow_mut()
                    .insert(Default::default()),
            }),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    Ok(tree)
}
