use core::fmt;

use serde::{Deserialize, Serialize};

use bitmask::bitmask;

use sdl2::mouse::MouseButton;

use crate::{
    animation::exponential,
    buffer::Buffer2D,
    color::{self, Color},
    debug_print,
    device::{MouseEventKind, MouseState},
    graphics::{text::TextOperation, Graphics},
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
        DrawText = (1 << 2),
        Hoverable = (1 << 3),
        Clickable = (1 << 4),
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UILayoutDirection {
    TopToBottom,
    #[default]
    LeftToRight,
}

impl fmt::Display for UILayoutDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UILayoutDirection::TopToBottom => "TopToBottom",
                UILayoutDirection::LeftToRight => "LeftToRight",
            }
        )
    }
}

pub static UI_BOX_HOT_COLOR: Color = color::RED;
pub static UI_BOX_ACTIVE_COLOR: Color = color::YELLOW;

pub static UI_BOX_TRANSITION_RATE: f32 = 15.0;

static UI_BOX_DEBUG_AUTOLAYOUT: bool = true;

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIBox's across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for box rendering.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIBox {
    pub id: String,
    pub key: UIKey,
    pub features: UIBoxFeatureMask,
    pub text_content: Option<String>,
    pub layout_direction: UILayoutDirection,
    pub parent_layout_direction: UILayoutDirection,
    pub semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
    pub styles: UIBoxStyles,
    #[serde(skip)]
    pub computed_relative_position: [f32; UI_2D_AXIS_COUNT], // Position relative to parent, in pixels.
    #[serde(skip)]
    pub computed_size: [f32; UI_2D_AXIS_COUNT], // Size in pixels.
    #[serde(skip)]
    pub global_bounds: ScreenExtent, // On-screen rectangle coordinates, in pixels.
    #[serde(skip)]
    pub hot: bool,
    #[serde(skip)]
    pub hot_transition: f32,
    #[serde(skip)]
    pub active: bool,
    #[serde(skip)]
    pub active_transition: f32,
    #[serde(skip)]
    pub last_read_at_frame: u32,
}

impl UIBox {
    pub fn new(
        mut id: String,
        features: UIBoxFeatureMask,
        layout_direction: UILayoutDirection,
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

        // Styles may have changed after the previous frame was rendered.

        let (mut fill_color, mut border_color) = (None, None);

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let styles = ctx.styles.borrow();

            if let Some(&color) = styles.fill_color.peek() {
                fill_color = Some(color);
            }

            if features.contains(UIBoxFeatureFlag::DrawBorder) {
                if let Some(&color) = styles.border_color.peek() {
                    border_color = Some(color);
                }
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
            layout_direction,
            semantic_sizes,
            styles,
            hot_transition: 1.0,
            active_transition: 1.0,
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

    pub fn update_hot_state(
        &mut self,
        seconds_since_last_update: f32,
        mouse_state: &mut MouseState,
    ) {
        self.hot = if self.features.contains(UIBoxFeatureFlag::Hoverable) && !self.key.is_null() {
            GLOBAL_UI_CONTEXT.with(|ctx| {
                let cache = ctx.cache.borrow();

                if let Some(ui_box_previous_frame) = cache.get(&self.key) {
                    // Check if our global mouse coordinates overlap this node's bounds.

                    let is_hot = ui_box_previous_frame
                        .global_bounds
                        .contains(mouse_state.position.0 as u32, mouse_state.position.1 as u32);

                    if is_hot {
                        // Resets hot animation transition (alpha) to zero.

                        self.hot_transition = 0.0;
                    } else {
                        // Updates hot animation transition alpha along a exponential curve.

                        self.hot_transition = exponential(
                            ui_box_previous_frame.hot_transition,
                            1.0,
                            seconds_since_last_update * UI_BOX_TRANSITION_RATE,
                        );
                    }

                    is_hot
                } else {
                    // We weren't rendered in previous frames, so we can't be hot yet.

                    false
                }
            })
        } else {
            // This node has no key (e.g., spacer, etc). Can't be hot.

            false
        };
    }

    pub fn update_active_state(
        &mut self,
        seconds_since_last_update: f32,
        mouse_state: &mut MouseState,
    ) {
        self.active = if self.features.contains(UIBoxFeatureFlag::Clickable) && !self.key.is_null()
        {
            GLOBAL_UI_CONTEXT.with(|ctx| {
                let cache = ctx.cache.borrow();

                if let Some(ui_box_previous_frame) = cache.get(&self.key) {
                    // Current active state will depend on our prior active
                    // state, if one exists.

                    let was_active = ui_box_previous_frame.active;

                    let is_active = if was_active {
                        // If we were active in the previous frame, we can only
                        // become inactive if the user released the left mouse
                        // button.

                        if let Some(event) = &mouse_state.button_event {
                            !matches!(
                                (event.button, event.kind),
                                (MouseButton::Left, MouseEventKind::Up)
                            )
                        } else {
                            // Otherwise, we remain active in this frame.

                            true
                        }
                    } else {
                        // If we weren't active in the previous frame, we can
                        // become active if the user presses their left mouse
                        // button while we are the hot element.

                        if let Some(event) = &mouse_state.button_event {
                            let did_transition_to_active = self.hot
                                && matches!(
                                    (event.button, event.kind),
                                    (MouseButton::Left, MouseEventKind::Down)
                                );

                            if did_transition_to_active {
                                mouse_state.button_event.take();
                            }

                            did_transition_to_active
                        } else {
                            false
                        }
                    };

                    if is_active {
                        // Resets active animation transition (alpha) to zero.

                        self.active_transition = 0.0;
                    } else {
                        // Updates active animation transition alpha along a exponential curve.

                        exponential(
                            ui_box_previous_frame.active_transition,
                            1.0,
                            seconds_since_last_update * UI_BOX_TRANSITION_RATE,
                        );
                    }

                    is_active
                } else {
                    // We weren't rendered in previous frames, so we can't be active yet.

                    false
                }
            })
        } else {
            // This node has no key (e.g., spacer, etc). Can't be active.

            false
        };
    }

    pub fn render(&self, target: &mut Buffer2D) -> Result<(), String> {
        let (x, y) = self.get_pixel_coordinates();

        let (width, height) = self.get_computed_pixel_size();

        let fill_color = if self.active_transition < 0.999 {
            let end = match self.styles.fill_color {
                Some(color) => color,
                None => Default::default(),
            };

            let with_hot = UI_BOX_HOT_COLOR.lerp_linear(end, self.hot_transition);

            Some(UI_BOX_ACTIVE_COLOR.lerp_linear(with_hot, self.active_transition))
        } else if self.hot_transition < 0.999 {
            let end = match self.styles.fill_color {
                Some(color) => color,
                None => Default::default(),
            };

            Some(UI_BOX_HOT_COLOR.lerp_linear(end, self.hot_transition))
        } else {
            self.styles.fill_color
        };

        let border_color = if UI_BOX_DEBUG_AUTOLAYOUT {
            Some(&color::RED)
        } else {
            self.styles.border_color.as_ref()
        };

        Graphics::rectangle(
            target,
            x,
            y,
            width,
            height,
            fill_color.as_ref(),
            border_color,
        );

        if self.features.contains(UIBoxFeatureFlag::DrawText) {
            let text_content = self.text_content.as_ref().expect("Called UIBox::render() with `UIBoxFeatureFlag::DrawText` when `text_content` is `None`!");

            GLOBAL_UI_CONTEXT.with(|ctx| {
                let mut text_cache = ctx.text_cache.borrow_mut();
                let font_info = ctx.font_info.borrow();
                let mut font_cache_rc = ctx.font_cache.borrow_mut();
                let font_cache = font_cache_rc.as_mut().expect("Found a UIBox with `DrawText` feature enabled when `GLOBAL_UI_CONTEXT.font_cache` is `None`!");

                Graphics::text(
                    target,
                    font_cache,
                    Some(&mut text_cache),
                    &font_info,
                    &TextOperation {
                        text: text_content,
                        x,
                        y,
                        color: color::WHITE
                    }
                ).unwrap();
            });
        }

        Ok(())
    }
}

impl fmt::Display for UIBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UIBox(id=\"{}\", hash={})", self.id, self.key)
    }
}
