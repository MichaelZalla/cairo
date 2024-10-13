use cairo::ui::ui_box::tree::UIBoxTree;

pub mod rasterization_options_panel;
pub mod render_options_panel;
pub mod settings_panel;
pub mod shader_options_panel;

pub trait PanelInstance {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String>;
}
