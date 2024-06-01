use core::fmt;

use serde::{Deserialize, Serialize};

use cairo::ui::{
    panel::{tree::PanelTree, Panel},
    ui_box::UILayoutDirection,
};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EditorPanelType {
    #[default]
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
                EditorPanelType::Outline => "Outline",
                EditorPanelType::Viewport3D => "Viewport3D",
                EditorPanelType::AssetBrowser => "AssetBrowser",
                EditorPanelType::Console => "Console",
                EditorPanelType::Inspector => "Inspector",
                EditorPanelType::FileSystem => "FileSystem",
            }
        )
    }
}

pub fn build_main_window_panel_tree<'a>() -> Result<PanelTree<'a, EditorPanelType>, String> {
    let mut tree = PanelTree::with_root(Panel {
        path: "root".to_string(),
        alpha_split: 1.0,
        panel_type: Some(EditorPanelType::Outline),
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
            Some(EditorPanelType::Outline),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Root > Left > Bottom (FileSystem).

    tree.push(
        "Bottom",
        Panel::new(
            0.5,
            Some(EditorPanelType::FileSystem),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to root.

    tree.pop_parent()?;

    // Root > Middle (3D Viewport, Console).

    tree.push_parent(
        "Middle",
        Panel::new(0.6, None, UILayoutDirection::TopToBottom),
    )?;

    tree.push_parent("Top", Panel::new(0.7, None, UILayoutDirection::TopToBottom))?;

    tree.push_parent("Top", Panel::new(0.5, None, UILayoutDirection::LeftToRight))?;

    tree.push(
        "Left",
        Panel::new(
            0.5,
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Right",
        Panel::new(
            0.5,
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.pop_parent()?;

    tree.push_parent(
        "Bottom",
        Panel::new(0.5, None, UILayoutDirection::LeftToRight),
    )?;

    tree.push(
        "Left",
        Panel::new(
            0.5,
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Right",
        Panel::new(
            0.5,
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.pop_parent()?;

    tree.pop_parent()?;

    tree.push(
        "Bottom",
        Panel::new(
            0.3,
            Some(EditorPanelType::Console),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to root.

    tree.pop_parent()?;

    // Root > Right (Inspector).

    tree.push_parent(
        "Right",
        Panel::new(
            0.2,
            Some(EditorPanelType::Inspector),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    Ok(tree)
}
