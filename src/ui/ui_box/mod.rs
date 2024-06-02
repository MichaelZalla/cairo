use std::{cell::RefCell, fmt, mem, rc::Rc};

use serde::{Deserialize, Serialize};

use bitmask::bitmask;

use sdl2::mouse::MouseButton;

use crate::{
    animation::{exponential, lerp},
    buffer::Buffer2D,
    color::{self, Color},
    debug_print,
    device::mouse::{cursor::MouseCursorKind, MouseEventKind, MouseState},
    graphics::{text::TextOperation, Graphics},
    ui::context::GLOBAL_UI_CONTEXT,
};

use interaction::UIBoxInteraction;
use key::UIKey;
use styles::UIBoxStyles;
use tree::FocusedTransitionInfo;

use super::{extent::ScreenExtent, tree::node::Node, UISizeWithStrictness, UI_2D_AXIS_COUNT};

pub mod interaction;
pub mod key;
pub mod styles;
pub mod tree;
pub mod utils;

bitmask! {
    #[derive(Default, Debug, Serialize, Deserialize)]
    pub mask UIBoxFeatureMask: u32 where flags UIBoxFeatureFlag {
        Null = 0,
        DrawFill = (1 << 0),
        DrawBorder = (1 << 1),
        EmbossAndDeboss = (1 << 2),
        DrawText = (1 << 3),
        SkipTextCaching = (1 << 4),
        Hoverable = (1 << 5),
        Clickable = (1 << 6),
        DrawChildDividers = (1 << 7),
        Resizable = (1 << 8),
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

pub static UI_BOX_HOT_TRANSITION_RATE: f32 = 15.0;
pub static UI_BOX_ACTIVE_TRANSITION_RATE: f32 = 15.0;
pub static UI_BOX_FOCUSED_TRANSITION_RATE: f32 = 5.0;

pub static UI_DIVIDER_CURSOR_SNAP_EPSILON: i32 = 3;

static UI_BOX_DEBUG_AUTOLAYOUT: bool = false;

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
    pub focused: bool,
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

        let (mut fill_color, mut border_color, mut text_color) = (None, None, None);

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

            if features.contains(UIBoxFeatureFlag::DrawText) {
                if let Some(&color) = styles.text_color.peek() {
                    text_color = Some(color);
                }
            }
        });

        let styles = UIBoxStyles {
            fill_color,
            border_color,
            text_color,
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
        interaction_result: &UIBoxInteraction,
    ) {
        self.hot = if self.features.contains(UIBoxFeatureFlag::Hoverable) && !self.key.is_null() {
            GLOBAL_UI_CONTEXT.with(|ctx| {
                let cache = ctx.cache.borrow();

                if let Some(ui_box_previous_frame) = cache.get(&self.key) {
                    // Check if our global mouse coordinates overlap this node's bounds.

                    let is_hot = interaction_result.mouse_interaction_in_bounds.is_hovering;

                    if is_hot {
                        // Resets hot animation transition (alpha) to zero.

                        self.hot_transition = 0.0;
                    } else {
                        // Updates hot animation transition alpha along a exponential curve.

                        self.hot_transition = exponential(
                            ui_box_previous_frame.hot_transition,
                            1.0,
                            seconds_since_last_update * UI_BOX_HOT_TRANSITION_RATE,
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
    ) -> bool {
        let mut did_transition_to_active = false;

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
                            did_transition_to_active = self.hot
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
                            seconds_since_last_update * UI_BOX_ACTIVE_TRANSITION_RATE,
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

        did_transition_to_active
    }

    pub fn update_focused_state(
        &mut self,
        new_focused_key: &Option<UIKey>,
        focused_transition_info: &mut FocusedTransitionInfo,
        seconds_since_last_update: f32,
    ) {
        GLOBAL_UI_CONTEXT.with(|ctx| {
            let cache = ctx.cache.borrow();

            if let Some(ui_box_previous_frame) = cache.get(&self.key) {
                match new_focused_key {
                    Some(key) => {
                        if self.key == *key {
                            if !ui_box_previous_frame.focused {
                                self.focused = true;

                                focused_transition_info.transition = 0.0;
                            }
                        } else {
                            if ui_box_previous_frame.focused {
                                focused_transition_info.from_rect =
                                    ui_box_previous_frame.global_bounds;
                            }

                            self.focused = false;
                        }
                    }
                    None => {
                        // Current active state will depend on our prior active
                        // state, if one exists.

                        let was_focused = ui_box_previous_frame.focused;

                        self.focused = was_focused;

                        if self.focused {
                            focused_transition_info.transition = exponential(
                                focused_transition_info.transition,
                                1.0,
                                seconds_since_last_update * UI_BOX_FOCUSED_TRANSITION_RATE,
                            );

                            focused_transition_info.current_rect.left = lerp(
                                focused_transition_info.from_rect.left as f32,
                                ui_box_previous_frame.global_bounds.left as f32,
                                focused_transition_info.transition,
                            )
                                as u32;

                            focused_transition_info.current_rect.top = lerp(
                                focused_transition_info.from_rect.top as f32,
                                ui_box_previous_frame.global_bounds.top as f32,
                                focused_transition_info.transition,
                            )
                                as u32;

                            focused_transition_info.current_rect.right = lerp(
                                focused_transition_info.from_rect.right as f32,
                                ui_box_previous_frame.global_bounds.right as f32,
                                focused_transition_info.transition,
                            )
                                as u32;

                            focused_transition_info.current_rect.bottom = lerp(
                                focused_transition_info.from_rect.bottom as f32,
                                ui_box_previous_frame.global_bounds.bottom as f32,
                                focused_transition_info.transition,
                            )
                                as u32;

                            if focused_transition_info.transition > 0.999 {
                                focused_transition_info.from_rect =
                                    ui_box_previous_frame.global_bounds;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn render_preorder(&self, target: &mut Buffer2D) -> Result<(), String> {
        let (x, y) = self.get_pixel_coordinates();
        let (width, height) = self.get_computed_pixel_size();

        let end = match self.styles.fill_color {
            Some(color) => color,
            None => Default::default(),
        };

        let fill_color = if self.active_transition < 0.999 {
            let with_hot = UI_BOX_HOT_COLOR.lerp_linear(end, self.hot_transition);

            Some(UI_BOX_ACTIVE_COLOR.lerp_linear(with_hot, self.active_transition))
        } else if self.hot_transition < 0.999 {
            Some(UI_BOX_HOT_COLOR.lerp_linear(end, 0.5 + self.hot_transition / 2.0))
        } else {
            self.styles.fill_color
        };

        Graphics::rectangle(target, x, y, width, height, fill_color.as_ref(), None);

        if self.features.contains(UIBoxFeatureFlag::DrawText) {
            let text_content = self.text_content.as_ref().expect("Called UIBox::render() with `UIBoxFeatureFlag::DrawText` when `text_content` is `None`!");

            let text_color = match self.styles.text_color {
                Some(color) => color,
                None => Default::default(),
            };

            GLOBAL_UI_CONTEXT.with(|ctx| {
                let mut text_cache = ctx.text_cache.borrow_mut();
                let font_info = ctx.font_info.borrow();
                let mut font_cache_rc = ctx.font_cache.borrow_mut();
                let font_cache = font_cache_rc.as_mut().expect("Found a UIBox with `DrawText` feature enabled when `GLOBAL_UI_CONTEXT.font_cache` is `None`!");

                Graphics::text(
                    target,
                    font_cache,
                    if self.features.contains(UIBoxFeatureFlag::SkipTextCaching) { None } else { Some(&mut text_cache) },
                    &font_info,
                    &TextOperation {
                        text: text_content,
                        x,
                        y,
                        color: text_color
                    }
                ).unwrap();
            });
        }

        Ok(())
    }

    pub fn render_postorder(
        &self,
        children: &Vec<Rc<RefCell<Node<UIBox>>>>,
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        if self.features.contains(UIBoxFeatureFlag::DrawChildDividers) && !children.is_empty() {
            let divider_color = match self.styles.border_color {
                Some(color) => color,
                None => Default::default(),
            };

            for (child_index, child) in children.iter().enumerate() {
                if child_index == 0 {
                    continue;
                }

                let child_node = &*child.borrow();
                let child_ui_box = &child_node.data;

                let (x1, y1, x2, y2) = match self.layout_direction {
                    UILayoutDirection::TopToBottom => {
                        // Draw a horizontal line across the top of this child.

                        (
                            child_ui_box.global_bounds.left as i32,
                            child_ui_box.global_bounds.top as i32,
                            child_ui_box.global_bounds.right as i32,
                            child_ui_box.global_bounds.top as i32,
                        )
                    }
                    UILayoutDirection::LeftToRight => {
                        // Draw a vertical line along the left of this child.

                        (
                            child_ui_box.global_bounds.left as i32,
                            child_ui_box.global_bounds.top as i32,
                            child_ui_box.global_bounds.left as i32,
                            child_ui_box.global_bounds.bottom as i32,
                        )
                    }
                };

                Graphics::line(target, x1, y1, x2, y2, &divider_color);

                if self.features.contains(UIBoxFeatureFlag::Resizable) {
                    // Set cursor if nearby.

                    GLOBAL_UI_CONTEXT.with(|ctx| {
                        let mouse_position = ctx.input_events.borrow().mouse.position;

                        let (mouse_x, mouse_y) = (mouse_position.0, mouse_position.1);

                        match self.layout_direction {
                            UILayoutDirection::TopToBottom => {
                                if (mouse_y - y1).abs() < UI_DIVIDER_CURSOR_SNAP_EPSILON
                                    && (x1..x2 + 1).contains(&mouse_x)
                                {
                                    GLOBAL_UI_CONTEXT.with(|ctx| {
                                        *ctx.cursor_kind.borrow_mut() = MouseCursorKind::DragUpDown;
                                    });
                                }
                            }
                            UILayoutDirection::LeftToRight => {
                                if (mouse_x - x1).abs() < UI_DIVIDER_CURSOR_SNAP_EPSILON
                                    && (y1..y2 + 1).contains(&mouse_y)
                                {
                                    GLOBAL_UI_CONTEXT.with(|ctx| {
                                        *ctx.cursor_kind.borrow_mut() =
                                            MouseCursorKind::DragLeftRight;
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }

        if self.features.contains(UIBoxFeatureFlag::DrawBorder)
            || self.features.contains(UIBoxFeatureFlag::DrawChildDividers)
        {
            let (x, y) = self.get_pixel_coordinates();
            let (width, height) = self.get_computed_pixel_size();

            if self.features.contains(UIBoxFeatureFlag::DrawBorder) {
                let border_color = if UI_BOX_DEBUG_AUTOLAYOUT {
                    Some(&color::BLUE)
                } else if self.styles.border_color.is_some() {
                    self.styles.border_color.as_ref()
                } else {
                    None
                };

                Graphics::rectangle(target, x, y, width, height, None, border_color);
            }

            if self.features.contains(UIBoxFeatureFlag::EmbossAndDeboss) {
                let (mut top_left, mut bottom_right) = (color::WHITE, color::BLACK);

                // Emboss-deboss.

                if self.active {
                    mem::swap(&mut top_left, &mut bottom_right);
                }

                let (x1, y1, x2, y2) = (
                    x as i32,
                    y as i32,
                    (x + width - 1) as i32,
                    (y + height - 1) as i32,
                );

                // Top edge.
                Graphics::line(target, x1, y1, x2, y1, &top_left);

                // Left edge.
                Graphics::line(target, x1, y1, x1, y2, &top_left);

                // Bottom edge.
                Graphics::line(target, x1, y2, x2, y2, &bottom_right);

                // Right edge.
                Graphics::line(target, x2, y1, x2, y2, &bottom_right);
            }
        }

        Ok(())
    }
}

impl fmt::Display for UIBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UIBox(id=\"{}\", hash={})", self.id, self.key)
    }
}
