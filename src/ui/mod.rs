use serde::{Deserialize, Serialize};

use crate::vec::vec2::Vec2;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UISize {
    #[default]
    Null,
    Pixels(u32),
    TextContent,
    PercentOfParent(f32),
    ChildrenSum,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UISizeWithStrictness {
    size: UISize,
    strictness: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum UI2DAxis {
    #[default]
    X,
    Y,
}

const UI_2D_AXIS_COUNT: usize = 2;

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIWidgets across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for widget rendering.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIWidget {
    // Auto-layout inputs
    semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],

    // Auto-layout outputs
    #[serde(skip)]
    computed_relative_position: [f32; UI_2D_AXIS_COUNT], // Position relative to parent, in pixels.

    #[serde(skip)]
    computed_size: [f32; UI_2D_AXIS_COUNT], // Size in pixels.

    #[serde(skip)]
    rect: (Vec2, Vec2), // On-screen rectangle coordinates, in pixels.
}

#[derive(Default, Debug, Clone)]
pub struct UIWidgetStack {
    stack: Vec<UIWidget>,
}

impl UIWidgetStack {
    pub fn push(&mut self, widget: UIWidget) {
        self.stack.push(widget)
    }

    pub fn pop(&mut self) -> Option<UIWidget> {
        self.stack.pop()
    }
}

pub struct UIContext {
    pub stack: UIWidgetStack,
}

impl UIContext {}
