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
}

impl PostDeserialize for Empty {
    fn post_deserialize(&mut self) {
        // Do nothing.
    }
}
