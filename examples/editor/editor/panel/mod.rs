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
    pub fn new(panel_type: Option<EditorPanelType>, layout_direction: UILayoutDirection) -> Self {
        Self {
            path: "".to_string(),
            alpha_split: 1.0,
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

pub fn build_panel_tree<'a>() -> Result<EditorPanelTree<'a>, String> {
    let mut tree = EditorPanelTree::with_root(EditorPanel {
        path: "main".to_string(),
        alpha_split: 1.0,
        panel_type: Some(EditorPanelType::Outline),
        layout_direction: UILayoutDirection::LeftToRight,
    });

    // Main > Left.

    tree.push_parent(
        "Left",
        EditorPanel::new(None, UILayoutDirection::TopToBottom),
    )?;

    // Main > Left > Top (Outline, 3D Viewport, Game).

    tree.push_parent(
        "Top",
        EditorPanel::new(None, UILayoutDirection::LeftToRight),
    )?;

    // Main > Left > Top > Left (Outline).

    tree.push(
        "Left",
        EditorPanel::new(
            Some(EditorPanelType::Outline),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Main > Left > Top > Middle (3D Viewport).

    tree.push(
        "Middle",
        EditorPanel::new(
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Main > Left > Top > Right (Game Viewport).

    tree.push(
        "Right",
        EditorPanel::new(
            Some(EditorPanelType::Viewport3D),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    // Back to Main > Left.

    tree.pop_parent()?;

    // Main > Left > Bottom (Assets, Console).

    tree.push(
        "Bottom",
        EditorPanel::new(
            Some(EditorPanelType::Console),
            UILayoutDirection::LeftToRight,
        ),
    )?;

    // Back to Main.

    tree.pop_parent()?;

    // Main > Right (Inspector).

    tree.push_parent(
        "Right",
        EditorPanel::new(
            Some(EditorPanelType::Inspector),
            UILayoutDirection::TopToBottom,
        ),
    )?;

    Ok(tree)
}
