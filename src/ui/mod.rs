use core::fmt;

use serde::{Deserialize, Serialize};

pub mod extent;
pub mod tree;
pub mod ui_box;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UISize {
    #[default]
    Null,
    Pixels(u32),
    TextContent,
    PercentOfParent(f32),
    ChildrenSum,
}

impl fmt::Display for UISize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UISize::Null => "Null",
                UISize::Pixels(_) => "Pixels",
                UISize::TextContent => "TextContent",
                UISize::PercentOfParent(_) => "PercentOfParent",
                UISize::ChildrenSum => "ChildrenSum",
            }
        )
    }
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
            "{}",
            match self {
                UI2DAxis::X => "X",
                UI2DAxis::Y => "Y",
            }
        )
    }
}

pub const UI_2D_AXIS_COUNT: usize = 2;
