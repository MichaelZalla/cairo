use serde::{Deserialize, Serialize};

use cairo::{
    serde::PostDeserialize,
    ui::{fastpath::text::text, ui_box::tree::UIBoxTree},
};

use super::PanelInstance;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AssetBrowserPanel {}

impl PostDeserialize for AssetBrowserPanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for AssetBrowserPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text(String::new(), "Asset Browser".to_string()))?;

        Ok(())
    }
}
