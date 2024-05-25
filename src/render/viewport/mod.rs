#[derive(Default, Debug, Copy, Clone)]
pub struct RenderViewport {
    pub width: u32,
    pub width_over_2: f32,
    pub height: u32,
    pub height_over_2: f32,
}
