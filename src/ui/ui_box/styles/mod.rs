use serde::{Deserialize, Serialize};

use crate::color::Color;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIBoxStylesMap<T: Default + Clone> {
    pub fill_color: T,
    pub border_color: T,
    pub text_color: T,
}

pub type UIBoxStyles = UIBoxStylesMap<Option<Color>>;
