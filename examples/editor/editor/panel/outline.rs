use serde::{Deserialize, Serialize};

use cairo::{
    serde::PostDeserialize,
    ui::{fastpath::text::text, ui_box::tree::UIBoxTree},
};

use super::PanelInstance;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OutlinePanel {}

impl PostDeserialize for OutlinePanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for OutlinePanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text(String::new(), "Outline".to_string()))?;

        Ok(())
    }
}
