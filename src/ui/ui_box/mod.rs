use core::fmt;

use serde::{Deserialize, Serialize};

use bitmask::bitmask;

use crate::{
    buffer::Buffer2D,
    debug_print,
    graphics::Graphics,
    ui::context::{UIBoxStyles, GLOBAL_UI_CONTEXT},
};

use self::key::UIKey;

use super::{extent::ScreenExtent, UISizeWithStrictness, UI_2D_AXIS_COUNT};

pub mod key;

bitmask! {
    #[derive(Default, Debug, Serialize, Deserialize)]
    pub mask UIBoxFeatureMask: u32 where flags UIBoxFeatureFlag {
        DrawFill = (1 << 0),
        DrawBorder = (1 << 1),
    }
}

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIBox's across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for box rendering.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIBox {
    pub id: String,
    pub key: UIKey,
    pub features: UIBoxFeatureMask,
    pub semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
    pub styles: UIBoxStyles,
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

impl UIBox {
    pub fn new(
        mut id: String,
        features: UIBoxFeatureMask,
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

        let (mut fill_color, mut border_color) = (None, None);

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let styles = ctx.styles.borrow();

            if let Some(&color) = styles.fill_color.peek() {
                fill_color = Some(color);
            }

            if let Some(&color) = styles.border_color.peek() {
                border_color = Some(color);
            }
        });

        let styles = UIBoxStyles {
            fill_color,
            border_color,
        };

        let ui_box = Self {
            id,
            key,
            features,
            semantic_sizes,
            styles,
            ..Default::default()
        };

        debug_print!("Created {}", ui_box);

        #[allow(clippy::let_and_return)]
        ui_box
    }

    pub fn get_pixel_coordinates(&self) -> (u32, u32) {
        (self.global_bounds.left, self.global_bounds.top)
    }

    pub fn get_computed_pixel_size(&self) -> (u32, u32) {
        (self.computed_size[0] as u32, self.computed_size[1] as u32)
    }

    pub fn render(&self, target: &mut Buffer2D) -> Result<(), String> {
        let (x, y) = self.get_pixel_coordinates();

        let (width, height) = self.get_computed_pixel_size();

        Graphics::rectangle(
            target,
            x,
            y,
            width,
            height,
            self.styles.fill_color.as_ref(),
            self.styles.border_color.as_ref(),
        );

        Ok(())
    }
}

impl fmt::Display for UIBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UIBox(id=\"{}\", hash={})", self.id, self.key)
    }
}
