use std::cell::RefCell;

use cairo::{
    resource::arena::Arena,
    ui::{panel::PanelRenderCallback, ui_box::tree::UIBoxTree},
};

use camera_attributes_panel::CameraAttributesPanel;
use rasterization_options_panel::RasterizationOptionsPanel;
use render_options_panel::RenderOptionsPanel;
use scene_graph_panel::SceneGraphPanel;
use settings_panel::SettingsPanel;
use shader_options_panel::ShaderOptionsPanel;

pub mod camera_attributes_panel;
pub mod rasterization_options_panel;
pub mod render_options_panel;
pub mod scene_graph_panel;
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
    pub scene_graph: &'static RefCell<Arena<SceneGraphPanel>>,
}

macro_rules! leak_arena {
    ($T: ident) => {{
        Box::leak(Box::new(RefCell::new(Arena::<$T>::new())))
    }};
}

impl Default for PanelArenas {
    fn default() -> Self {
        Self {
            settings: leak_arena!(SettingsPanel),
            render_options: leak_arena!(RenderOptionsPanel),
            shader_options: leak_arena!(ShaderOptionsPanel),
            rasterization_options: leak_arena!(RasterizationOptionsPanel),
            camera_attributes: leak_arena!(CameraAttributesPanel),
            scene_graph: leak_arena!(SceneGraphPanel),
        }
    }
}

pub struct PanelRenderCallbacks {
    pub settings: PanelRenderCallback,
    pub render_options: PanelRenderCallback,
    pub shader_options: PanelRenderCallback,
    pub rasterization_options: PanelRenderCallback,
    pub camera_attributes: PanelRenderCallback,
    pub scene_graph: PanelRenderCallback,
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
