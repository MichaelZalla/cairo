use core::fmt;

use serde::{Deserialize, Serialize};

use bitmask::bitmask;

use crate::vec::vec2::Vec2;

use super::{UISizeWithStrictness, UI_2D_AXIS_COUNT};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIKey {
    hash: Option<String>,
}

impl fmt::Display for UIKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UIKey(hash={})",
            if let Some(hash) = &self.hash {
                format!("\"{}\"", hash)
            } else {
                "None".to_string()
            }
        )
    }
}

bitmask! {
    #[derive(Default, Debug, Serialize, Deserialize)]
    pub mask UIWidgetFeatureMask: u32 where flags UIWidgetFeatureFlag {
        DrawFill = (1 << 0),
        DrawBorder = (1 << 1),
    }
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
    pub global_bounds: [Vec2; 2], // On-screen rectangle coordinates, in pixels.
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

        let hash;

        if id_split_strings.len() == 1 {
            hash = None;
        } else {
            id = id_split_strings[0].to_string();
            hash = Some(id_split_strings[1].to_string());
        };

        let key = UIKey { hash };

        let widget = Self {
            id,
            key,
            features,
            semantic_sizes,
            ..Default::default()
        };

        println!("Created {}", widget);

        widget
    }

    pub fn get_pixel_coordinates(&self) -> (u32, u32) {
        (
            self.global_bounds[0].x as u32,
            self.global_bounds[0].y as u32,
        )
    }

    pub fn get_computed_pixel_size(&self) -> (u32, u32) {
        (self.computed_size[0] as u32, self.computed_size[1] as u32)
    }
}

impl fmt::Display for UIWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UIWidget(id=\"{}\", hash={})", self.id, self.key)
    }
}
