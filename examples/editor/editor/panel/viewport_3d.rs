use std::rc::Rc;
use std::{cell::RefCell, fmt::Debug};

use cairo::buffer::Buffer2D;
use cairo::scene::graph::SceneGraphRenderOptions;
use cairo::software_renderer::SoftwareRenderer;
use cairo::ui::extent::ScreenExtent;
use cairo::vec::vec4::Vec4;
use serde::{Deserialize, Serialize};

use cairo::{
    buffer::framebuffer::Framebuffer, resource::handle::Handle, serde::PostDeserialize,
    ui::ui_box::tree::UIBoxTree,
};

use crate::EDITOR_SCENE_CONTEXT;

use super::PanelInstance;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Viewport3DPanel {
    #[serde(skip)]
    renderer: Option<Rc<RefCell<SoftwareRenderer>>>,
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
    pub fn new(renderer: Rc<RefCell<SoftwareRenderer>>, active_camera: Handle) -> Self {
        let mut framebuffer = Framebuffer::new(100, 100);

        framebuffer.complete(0.3, 100.0);

        Self {
            renderer: Some(renderer),
            framebuffer: Rc::new(RefCell::new(framebuffer)),
            active_camera,
        }
    }
}

impl PanelInstance for Viewport3DPanel {
    fn render(&mut self, _tree: &mut UIBoxTree) -> Result<(), String> {
        Ok(())
    }

    fn custom_render_callback(
        &mut self,
        extent: &ScreenExtent,
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        let (panel_width, panel_height) = (extent.right - extent.left, extent.bottom - extent.top);

        {
            let mut framebuffer = (*self.framebuffer).borrow_mut();

            if framebuffer.width != panel_width || framebuffer.height != panel_height {
                framebuffer.resize(panel_width, panel_height, false);
            }
        }

        {
            let renderer_rc = self.renderer.as_ref().unwrap();
            let mut renderer = (*renderer_rc).borrow_mut();

            renderer.bind_framebuffer(Some(self.framebuffer.clone()));
        }

        EDITOR_SCENE_CONTEXT.with(|scene_context| {
            let resources = scene_context.resources.borrow();
            let scenes = scene_context.scenes.borrow();
            let scene = &scenes[0];

            let renderer_rc = self.renderer.as_ref().unwrap();

            {
                let renderer = (*renderer_rc).borrow_mut();
                let camera_arena = resources.camera.borrow();

                if let Ok(entry) = camera_arena.get(&self.active_camera) {
                    let camera = &entry.item;

                    let camera_view_inverse_transform = camera.get_view_inverse_transform();

                    let mut shader_context = (*renderer.shader_context).borrow_mut();

                    shader_context
                        .set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

                    shader_context.set_view_inverse_transform(camera_view_inverse_transform);

                    shader_context.set_projection(camera.get_projection());
                }
            }

            scene
                .render(
                    &resources,
                    renderer_rc.as_ref(),
                    Some(SceneGraphRenderOptions {
                        camera: Some(self.active_camera),
                        ..Default::default()
                    }),
                )
                .unwrap();
        });

        {
            let framebuffer = (*self.framebuffer).borrow_mut();

            if let Some(color_buffer_rc) = &framebuffer.attachments.color {
                let color_buffer = (*color_buffer_rc).borrow();

                target.blit_from(extent.left, extent.top, &color_buffer);
            }
        }

        Ok(())
    }
}