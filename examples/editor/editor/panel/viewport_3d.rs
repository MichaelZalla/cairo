use std::rc::Rc;
use std::{cell::RefCell, fmt::Debug};

use cairo::ui::ui_box::utils::text_box;
use serde::{Deserialize, Serialize};

use cairo::{
    buffer::framebuffer::Framebuffer, resource::handle::Handle, serde::PostDeserialize,
    ui::ui_box::tree::UIBoxTree,
};

use super::PanelInstance;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Viewport3DPanel {
    #[serde(skip)]
    framebuffer: Rc<RefCell<Framebuffer>>,
    active_camera: Handle,
}

impl Debug for Viewport3DPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Viewport3DPanel")
            // .field("renderer", &self.renderer)
            .field("framebuffer", &self.framebuffer)
            .field("active_camera", &self.active_camera)
            .finish()
    }
}

impl PostDeserialize for Viewport3DPanel {
    fn post_deserialize(&mut self) {}
}

impl Viewport3DPanel {
    pub fn new(active_camera: Handle) -> Self {
        let mut framebuffer = Framebuffer::new(100, 100);

        framebuffer.complete(0.3, 100.0);

        Self {
            framebuffer: Rc::new(RefCell::new(framebuffer)),
            active_camera,
        }
    }
}

impl PanelInstance for Viewport3DPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        tree.push(text_box(String::new(), "Viewport3D".to_string()))?;

        Ok(())
    }
}
