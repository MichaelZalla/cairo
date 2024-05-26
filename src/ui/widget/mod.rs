use core::fmt;

use serde::{Deserialize, Serialize};

use bitmask::bitmask;

use crate::{
    buffer::Buffer2D,
    color::{self, Color},
    debug_print,
    graphics::Graphics,
};

use self::key::UIKey;

use super::{UISizeWithStrictness, UI_2D_AXIS_COUNT};

pub mod key;

bitmask! {
    #[derive(Default, Debug, Serialize, Deserialize)]
    pub mask UIWidgetFeatureMask: u32 where flags UIWidgetFeatureFlag {
        DrawFill = (1 << 0),
        DrawBorder = (1 << 1),
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct ScreenExtent {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIWidgets across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for widget rendering.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIWidget {
    pub id: String,
    pub key: UIKey,
    pub features: UIWidgetFeatureMask,
    pub semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
    #[serde(skip)]
    pub computed_relative_position: [f32; UI_2D_AXIS_COUNT], // Position relative to parent, in pixels.
    #[serde(skip)]
    pub computed_size: [f32; UI_2D_AXIS_COUNT], // Size in pixels.
    #[serde(skip)]
    pub global_bounds: ScreenExtent, // On-screen rectangle coordinates, in pixels.
    #[serde(skip)]
    pub hot_transition: f32,
    #[serde(skip)]
    pub active_transition: f32,
    #[serde(skip)]
    pub last_read_at_frame: u32,
}

impl UIWidget {
    pub fn new(
        mut id: String,
        features: UIWidgetFeatureMask,
        semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
    ) -> Self {
        let id_split_str = id.split("__").collect::<Vec<&str>>();

        let id_split_strings = id_split_str
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>();

        let key = if id_split_strings.len() == 1 {
            Default::default()
        } else {
            id = id_split_strings[0].to_string();

            UIKey::from_string(id_split_strings[1].to_string())
        };

        let widget = Self {
            id,
            key,
            features,
            semantic_sizes,
            ..Default::default()
        };

        debug_print!("Created {}", widget);

        #[allow(clippy::let_and_return)]
        widget
    }

    pub fn get_pixel_coordinates(&self) -> (u32, u32) {
        (self.global_bounds.left, self.global_bounds.top)
    }

    pub fn get_computed_pixel_size(&self) -> (u32, u32) {
        (self.computed_size[0] as u32, self.computed_size[1] as u32)
    }

    pub fn render(
        &self,
        depth: usize,
        _frame_index: u32,
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        let (x, y) = self.get_pixel_coordinates();
        let (width, height) = self.get_computed_pixel_size();

        static COLOR_FOR_DEPTH: [Color; 4] = [color::YELLOW, color::BLUE, color::RED, color::GREEN];

        let fill_color = if self.features.contains(UIWidgetFeatureFlag::DrawFill) {
            Some(COLOR_FOR_DEPTH[depth])
        } else {
            None
        };
        let border_color = if self.features.contains(UIWidgetFeatureFlag::DrawBorder) {
            Some(color::BLACK)
        } else {
            None
        };

        Graphics::rectangle(target, x, y, width, height, fill_color, border_color);

        Ok(())
    }
}

impl fmt::Display for UIWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UIWidget(id=\"{}\", hash={})", self.id, self.key)
    }
}
