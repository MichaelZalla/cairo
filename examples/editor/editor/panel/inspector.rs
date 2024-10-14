use serde::{Deserialize, Serialize};

use cairo::{
    serde::PostDeserialize,
    ui::{fastpath::text::text, ui_box::tree::UIBoxTree},
};

use super::PanelInstance;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InspectorPanel {}

impl PostDeserialize for InspectorPanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for InspectorPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text(String::new(), "Inspector".to_string()))?;

        Ok(())
    }
}
