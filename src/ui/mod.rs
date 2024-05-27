use core::fmt;

use serde::{Deserialize, Serialize};

pub mod context;
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
    Primary,
    Secondary,
}

impl UI2DAxis {
    pub fn from_usize(i: usize) -> Self {
        if i == 0 {
            Self::Primary
        } else if i == 1 {
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
