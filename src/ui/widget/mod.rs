use serde::{Deserialize, Serialize};

use crate::vec::vec2::Vec2;

use super::{UISizeWithStrictness, UI_2D_AXIS_COUNT};

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIWidgets across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for widget rendering.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIWidget {
    pub id: String,

    // Auto-layout inputs
    pub semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],

    // Auto-layout outputs
    #[serde(skip)]
    pub computed_relative_position: [f32; UI_2D_AXIS_COUNT], // Position relative to parent, in pixels.

    #[serde(skip)]
    pub computed_size: [f32; UI_2D_AXIS_COUNT], // Size in pixels.

    #[serde(skip)]
    pub global_bounds: [Vec2; 2], // On-screen rectangle coordinates, in pixels.
}

impl UIWidget {
    pub fn new(id: String, semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT]) -> Self {
        Self {
            id,
            semantic_sizes,
            ..Default::default()
        }
    }
}
