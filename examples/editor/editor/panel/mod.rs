use core::fmt;

use serde::{Deserialize, Serialize};

use cairo::ui::{
    ui_box::{UIBox, UIBoxFeatureFlag, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

use tree::EditorPanelTree;

pub mod tree;

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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EditorPanel {
    pub path: String,
    // For this panel.
    pub alpha_split: f32,
    pub panel_type: Option<EditorPanelType>,
    // For child panels.
    pub layout_direction: UILayoutDirection,
}

impl fmt::Display for EditorPanel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Panel ({})", self.path)
    }
}

impl EditorPanel {
    pub fn new(
        alpha_split: f32,
        panel_type: Option<EditorPanelType>,
        layout_direction: UILayoutDirection,
    ) -> Self {
        Self {
            path: "".to_string(),
            alpha_split,
            panel_type,
            layout_direction,
        }
    }

    pub fn render(&self) -> UIBox {
        let panel_path = &self.path;
        let panel_path_cloned = panel_path.clone();
        let panel_path_components = panel_path_cloned.split(' ').collect::<Vec<&str>>();

        let panel_ui_box_id = panel_path_components.join("");
        let panel_ui_box_key_hash = panel_path_components
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>()
            .join("_");

        let text_content = self.panel_type.map(|panel_type| format!("{}", panel_type));

        let mut ui_box_feature_flags =
            UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::Hoverable | UIBoxFeatureFlag::Clickable;

        if text_content.is_some() {
            ui_box_feature_flags |= UIBoxFeatureFlag::DrawText
        }

        let mut panel_ui_box = UIBox::new(
            format!("{}__{}", panel_ui_box_id, panel_ui_box_key_hash),
            ui_box_feature_flags,
            self.layout_direction,
            [
                UISizeWithStrictness {
                    size: UISize::PercentOfParent(self.alpha_split),
                    strictness: 0.0,
                },
                UISizeWithStrictness {
                    size: UISize::PercentOfParent(1.0),
                    strictness: 1.0,
                },
            ],
        );

        panel_ui_box.text_content = text_content;

        panel_ui_box
    }
}

pub fn build_main_panel_tree<'a>() -> Result<EditorPanelTree<'a>, String> {
    let mut tree = EditorPanelTree::with_root(EditorPanel {
        path: "root".to_string(),
        alpha_split: 1.0,
        panel_type: Some(EditorPanelType::Outline),
        layout_direction: UILayoutDirection::LeftToRight,
    });

    // Root > Left.

    tree.push_parent(
        "Left",
        EditorPanel::new(0.33, None, UILayoutDirection::TopToBottom),
    )?;

    // Root > Left > Top (Scene).

    tree.push(
        "Top",
        EditorPanel::new(
            0.5,
            Some(EditorPanelType::Outline),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Root > Left > Bottom (FileSystem).

    tree.push(
        "Bottom",
        EditorPanel::new(
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
        EditorPanel::new(0.66, None, UILayoutDirection::TopToBottom),
    )?;

    tree.push(
        "Top",
        EditorPanel::new(
            0.7,
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    tree.push(
        "Bottom",
        EditorPanel::new(
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
        EditorPanel::new(
            0.33,
            Some(EditorPanelType::Inspector),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    Ok(tree)
}
