use core::fmt;

use serde::{Deserialize, Serialize};

use crate::vec::vec2::Vec2;

use self::tree::UIWidgetTree;

pub mod tree;

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
    pub size: UISize,
    pub strictness: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum UI2DAxis {
    #[default]
    X,
    Y,
}

impl fmt::Display for UI2DAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UI2DAxis-{}",
            match self {
                UI2DAxis::X => "X",
                UI2DAxis::Y => "Y",
            }
        )
    }
}

const UI_2D_AXIS_COUNT: usize = 2;

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
    computed_relative_position: [f32; UI_2D_AXIS_COUNT], // Position relative to parent, in pixels.

    #[serde(skip)]
    computed_size: [f32; UI_2D_AXIS_COUNT], // Size in pixels.

    #[serde(skip)]
    rect: [Vec2; 2], // On-screen rectangle coordinates, in pixels.
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

pub struct UIContext<'a> {
    pub tree: UIWidgetTree<'a>,
}

impl<'a> UIContext<'a> {}
