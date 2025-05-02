use crate::buffer::framebuffer::Framebuffer;

#[derive(Default, Debug, Copy, Clone)]
pub struct RenderViewport {
    pub width: u32,
    pub width_over_2: f32,
    pub height: u32,
    pub height_over_2: f32,
}

impl From<&Framebuffer> for RenderViewport {
    fn from(framebuffer: &Framebuffer) -> Self {
        Self {
            width: framebuffer.width,
            width_over_2: framebuffer.width as f32 / 2.0,
            height: framebuffer.height,
            height_over_2: framebuffer.height as f32 / 2.0,
        }
    }
}
