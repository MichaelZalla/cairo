use serde::{Deserialize, Serialize};

use cairo::{
    serde::PostDeserialize,
    ui::ui_box::{tree::UIBoxTree, utils::text_box},
};

use super::PanelInstance;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AssetBrowserPanel {}

impl PostDeserialize for AssetBrowserPanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for AssetBrowserPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text_box(String::new(), "AssetBrowser".to_string()))?;

        Ok(())
    }
}