use core::fmt;

use serde::{Deserialize, Serialize};

use cairo::ui::{
    panel::{tree::PanelTree, Panel, PanelMetadata},
    ui_box::UILayoutDirection,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum EditorPanelType {
    #[default]
    Null,
    Outline,
    Viewport3D,
    AssetBrowser,
    Console,
    Inspector,
    FileSystem,
}

impl fmt::Display for EditorPanelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EditorPanelType::Null => "EditorPanelType::Null",
                EditorPanelType::Outline => "EditorPanelType::Outline",
                EditorPanelType::Viewport3D => "EditorPanelType::Viewport3D",
                EditorPanelType::AssetBrowser => "EditorPanelType::AssetBrowser",
                EditorPanelType::Console => "EditorPanelType::Console",
                EditorPanelType::Inspector => "EditorPanelType::Inspector",
                EditorPanelType::FileSystem => "EditorPanelType::FileSystem",
            }
        )
    }
}

pub struct EditorPanelMetadataMap {
    pub outline: PanelMetadata<EditorPanelType>,
    pub viewport3d: PanelMetadata<EditorPanelType>,
    pub asset_browser: PanelMetadata<EditorPanelType>,
    pub console: PanelMetadata<EditorPanelType>,
    pub inspector: PanelMetadata<EditorPanelType>,
    pub file_system: PanelMetadata<EditorPanelType>,
}

pub fn build_main_window_panel_tree<'a>(
    panel_metadata_map: &EditorPanelMetadataMap,
) -> Result<PanelTree<'a, EditorPanelType>, String> {
    let mut tree = PanelTree::with_root(Panel {
        path: "root".to_string(),
        resizable: false,
        alpha_split: 1.0,
        panel_metadata: None,
        layout_direction: UILayoutDirection::LeftToRight,
    });

    // Root > Left.

    tree.push_parent(
        "Left",
        Panel::new(0.2, None, UILayoutDirection::TopToBottom),
    )?;

    // Root > Left > Top (Scene).

    tree.push(
        "Top",
        Panel::new(
            0.5,
            Some(panel_metadata_map.outline.clone()),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Root > Left > Bottom (FileSystem).

    tree.push(
        "Bottom",
        Panel::new(
            0.5,
            Some(panel_metadata_map.file_system.clone()),
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
            Some(panel_metadata_map.viewport3d.clone()),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Right",
        Panel::new(
            0.5,
            Some(panel_metadata_map.viewport3d.clone()),
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
            Some(panel_metadata_map.viewport3d.clone()),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Right",
        Panel::new(
            0.5,
            Some(panel_metadata_map.viewport3d.clone()),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to Root > Bottom > Middle > Top.

    tree.pop_parent()?;

    // Back to Root > Bottom > Middle.

    tree.pop_parent()?;

    // Root > Middle > Bottom.

    tree.push(
        "Bottom",
        Panel::new(
            0.3,
            Some(panel_metadata_map.console.clone()),
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
            Some(panel_metadata_map.inspector.clone()),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    Ok(tree)
}
