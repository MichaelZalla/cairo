use std::cell::RefCell;

use cairo::{
    resource::arena::Arena,
    ui::{panel::PanelRenderCallback, ui_box::tree::UIBoxTree},
};
use camera_attributes_panel::CameraAttributesPanel;
use rasterization_options_panel::RasterizationOptionsPanel;
use render_options_panel::RenderOptionsPanel;
use settings_panel::SettingsPanel;
use shader_options_panel::ShaderOptionsPanel;

pub mod camera_attributes_panel;
pub mod rasterization_options_panel;
pub mod render_options_panel;
pub mod settings_panel;
pub mod shader_options_panel;

pub trait PanelInstance {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String>;
}

pub struct PanelArenas {
    pub settings: &'static RefCell<Arena<SettingsPanel>>,
    pub render_options: &'static RefCell<Arena<RenderOptionsPanel>>,
    pub shader_options: &'static RefCell<Arena<ShaderOptionsPanel>>,
    pub rasterization_options: &'static RefCell<Arena<RasterizationOptionsPanel>>,
    pub camera_attributes: &'static RefCell<Arena<CameraAttributesPanel>>,
}

impl Default for PanelArenas {
    fn default() -> Self {
        Self {
            settings: Box::leak(Box::new(RefCell::new(Arena::<SettingsPanel>::new()))),
            render_options: Box::leak(Box::new(RefCell::new(Arena::<RenderOptionsPanel>::new()))),
            shader_options: Box::leak(Box::new(RefCell::new(Arena::<ShaderOptionsPanel>::new()))),
            rasterization_options: Box::leak(Box::new(RefCell::new(Arena::<
                RasterizationOptionsPanel,
            >::new()))),
            camera_attributes: Box::leak(Box::new(RefCell::new(
                Arena::<CameraAttributesPanel>::new(),
            ))),
        }
    }
}

pub struct PanelRenderCallbacks {
    pub settings: PanelRenderCallback,
    pub render_options: PanelRenderCallback,
    pub shader_options: PanelRenderCallback,
    pub rasterization_options: PanelRenderCallback,
    pub camera_attributes: PanelRenderCallback,
}

#[macro_export]
macro_rules! panel_render_callback {
    ($panel_arenas: ident, $panel_kind: ident) => {
        |panel_instance: &Handle, tree: &mut UIBoxTree| -> Result<(), String> {
            let mut arena = $panel_arenas.$panel_kind.borrow_mut();

            if let Ok(entry) = arena.get_mut(panel_instance) {
                let panel = &mut entry.item;

                panel.render(tree).unwrap();
            }

            Ok(())
        }
    };
}
