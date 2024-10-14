use serde::{Deserialize, Serialize};

use cairo::{
    serde::PostDeserialize,
    ui::{fastpath::text::text, ui_box::tree::UIBoxTree},
};

use super::PanelInstance;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPanel {}

impl PostDeserialize for FileSystemPanel {
    fn post_deserialize(&mut self) {}
}

impl PanelInstance for FileSystemPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text(String::new(), "File System".to_string()))?;

        Ok(())
    }
}
