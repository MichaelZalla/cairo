use serde::{Deserialize, Serialize};

use cairo::{
    serde::PostDeserialize,
    ui::ui_box::{tree::UIBoxTree, utils::text_box},
};

use super::PanelInstance;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ConsolePanel {}

impl PostDeserialize for ConsolePanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for ConsolePanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text_box(String::new(), "Console".to_string()))?;

        Ok(())
    }
}
