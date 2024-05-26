#[derive(Default, Debug, Copy, Clone)]
pub struct ScreenExtent {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl ScreenExtent {
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}
