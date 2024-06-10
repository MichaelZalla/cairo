use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ScreenExtent {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl fmt::Display for ScreenExtent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({},{}) -> ({},{})",
            self.left, self.top, self.right, self.bottom
        )
    }
}

impl ScreenExtent {
    pub fn new(position: (u32, u32), size: (u32, u32)) -> Self {
        Self {
            left: position.0,
            right: position.0 + size.0,
            top: position.1,
            bottom: position.1 + size.1,
        }
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}
