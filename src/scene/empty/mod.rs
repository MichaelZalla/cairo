use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::serde::PostDeserialize;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Empty(pub EmptyDisplayKind);

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EmptyDisplayKind {
    #[default]
    Axes,
    Arrow,
    Square,
    Cube,
    Circle(usize),
    Sphere(usize),
    Capsule(usize, f32),
}

impl PostDeserialize for Empty {
    fn post_deserialize(&mut self) {
        // Do nothing.
    }
}

impl Display for EmptyDisplayKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Axes => "axes",
                Self::Arrow => "arrow",
                Self::Square => "square",
                Self::Cube => "cube",
                Self::Circle(_) => "circle",
                Self::Sphere(_) => "sphere",
                Self::Capsule(_, _) => "capsule",
            }
        )
    }
}
