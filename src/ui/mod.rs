use core::fmt;

use serde::{Deserialize, Serialize};

pub mod context;
pub mod extent;
pub mod fastpath;
pub mod panel;
pub mod ui_box;
pub mod window;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UISize {
    #[default]
    Null,
    Pixels(u32),
    TextContent,
    PercentOfParent(f32),
    ChildrenSum,
    MaxOfSiblings,
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
                UISize::MaxOfSiblings => "MaxOfSiblings",
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
    Primary,
    Secondary,
}

impl From<usize> for UI2DAxis {
    fn from(axis_index: usize) -> Self {
        if axis_index == 0 {
            Self::Primary
        } else if axis_index == 1 {
            Self::Secondary
        } else {
            panic!()
        }
    }
}

impl fmt::Display for UI2DAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UI2DAxis::Primary => "Primary",
                UI2DAxis::Secondary => "Secondary",
            }
        )
    }
}

pub const UI_2D_AXIS_COUNT: usize = 2;
