use std::{
    cell::RefCell,
    fmt::{self, Debug},
    mem,
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use bitmask::bitmask;

use sdl2::mouse::MouseButton;

use crate::{
    animation::{exponential, lerp},
    buffer::Buffer2D,
    collections::tree::node::Node,
    color::{self, Color},
    device::mouse::{cursor::MouseCursorKind, MouseEventKind, MouseState},
    graphics::{text::TextOperation, Graphics},
    resource::handle::Handle,
    ui::context::GLOBAL_UI_CONTEXT,
};

use interaction::UIBoxInteraction;
use key::UIKey;
use styles::UIBoxStyles;
use tree::FocusedTransitionInfo;

use super::{extent::ScreenExtent, UISizeWithStrictness, UI_2D_AXIS_COUNT};

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
        ResizableMinExtentOnPrimaryAxis = (1 << 7),
        ResizableMaxExtentOnPrimaryAxis = (1 << 8),
        ResizableMinExtentOnSecondaryAxis = (1 << 9),
        ResizableMaxExtentOnSecondaryAxis = (1 << 10),
        DrawChildDividers = (1 << 11),
        DrawCustomRender = (1 << 12),
        MaskCircle = (1 << 13),
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

pub static UI_BOX_SPACER_ID: &str = "__SPACER__";

pub static UI_BOX_HOT_COLOR: Color = color::RED;
pub static UI_BOX_ACTIVE_COLOR: Color = color::YELLOW;

pub static UI_BOX_HOT_TRANSITION_RATE: f32 = 15.0;
pub static UI_BOX_ACTIVE_TRANSITION_RATE: f32 = 15.0;
pub static UI_BOX_FOCUSED_TRANSITION_RATE: f32 = 5.0;

pub static UI_DIVIDER_CURSOR_SNAP_EPSILON: i32 = 3;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UIBoxDragHandle {
    Left,
    Top,
    Bottom,
    Right,
}

pub type UIBoxCustomRenderCallback =
    fn(&Option<Handle>, &ScreenExtent, &mut Buffer2D) -> Result<(), String>;

pub type UIBoxCustomRenderCallbackWithContextHandle = (UIBoxCustomRenderCallback, Option<Handle>);

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIBox's across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for box rendering.
#[derive(Default, Clone, Serialize, Deserialize)]
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
    pub hot_drag_handle: Option<UIBoxDragHandle>,
    #[serde(skip)]
    pub active_drag_handle: Option<UIBoxDragHandle>,
    #[serde(skip)]
    pub last_read_at_frame: u32,
    #[serde(skip)]
    pub custom_render_callback: Option<UIBoxCustomRenderCallbackWithContextHandle>,
}

impl Debug for UIBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UIBox")
            .field("id", &self.id)
            .field("key", &self.key)
            .field("features", &self.features)
            .field("text_content", &self.text_content)
            .field("layout_direction", &self.layout_direction)
            .field("parent_layout_direction", &self.parent_layout_direction)
            .field("semantic_sizes", &self.semantic_sizes)
            .field("styles", &self.styles)
            .field(
                "computed_relative_position",
                &self.computed_relative_position,
            )
            .field("computed_size", &self.computed_size)
            .field("global_bounds", &self.global_bounds)
            .field("hot", &self.hot)
            .field("hot_transition", &self.hot_transition)
            .field("active", &self.active)
            .field("active_transition", &self.active_transition)
            .field("focused", &self.focused)
            .field("hot_drag_handle", &self.hot_drag_handle)
            .field("active_drag_handle", &self.active_drag_handle)
            .field("last_read_at_frame", &self.last_read_at_frame)
            // .field("custom_render_callback", &self.custom_render_callback)
            .finish()
    }
}

impl UIBox {
    pub fn new(
        id: String,
        mut features: UIBoxFeatureMask,
        layout_direction: UILayoutDirection,
        semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
        custom_render_callback: Option<UIBoxCustomRenderCallbackWithContextHandle>,
    ) -> Self {
        let key = UIKey::from_string(id.clone());

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

        if custom_render_callback.is_some() {
            features |= UIBoxFeatureFlag::DrawCustomRender;
        }

        Self {
            id,
            key,
            features,
            layout_direction,
            semantic_sizes,
            styles,
            hot_transition: 1.0,
            active_transition: 1.0,
            custom_render_callback,
            ..Default::default()
        }
    }

    pub fn is_spacer(&self) -> bool {
        self.id == UI_BOX_SPACER_ID
    }

    pub fn get_pixel_coordinates(&self) -> (u32, u32) {
        (self.global_bounds.left, self.global_bounds.top)
    }

    pub fn get_computed_pixel_size(&self) -> (u32, u32) {
        (self.computed_size[0] as u32, self.computed_size[1] as u32)
    }

    pub fn contains_screen_pixel(&self, x: i32, y: i32) -> bool {
        self.global_bounds.contains(x as u32, y as u32)
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

        let is_active_transitioning = self.hot_transition < 0.999;
        let is_hot_transitioning = self.hot_transition < 0.999;

        #[cfg(debug_assertions)]
        let draw_active_hover_indicators =
            GLOBAL_UI_CONTEXT.with(|ctx| ctx.debug.borrow().draw_active_hover_indicator);

        #[cfg(not(debug_assertions))]
        let draw_active_hover_indicators = false;

        let should_draw_fill = {
            let has_fill_feature = self.features.contains(UIBoxFeatureFlag::DrawFill);

            if draw_active_hover_indicators {
                has_fill_feature || is_active_transitioning || is_hot_transitioning
            } else {
                has_fill_feature
            }
        };

        if should_draw_fill {
            let fill_color = if draw_active_hover_indicators {
                let end = self.styles.fill_color.unwrap_or_default();

                if is_active_transitioning {
                    let with_hot = UI_BOX_HOT_COLOR.lerp_linear(end, self.hot_transition);

                    Some(UI_BOX_ACTIVE_COLOR.lerp_linear(with_hot, self.active_transition))
                } else if is_hot_transitioning {
                    Some(UI_BOX_HOT_COLOR.lerp_linear(end, 0.5 + self.hot_transition / 2.0))
                } else {
                    self.styles.fill_color
                }
            } else {
                self.styles.fill_color
            };

            if self.features.contains(UIBoxFeatureFlag::MaskCircle) {
                let radius = (width.min(height) as f32 / 2.0).floor();
                let center = (x + width / 2, y + height / 2);

                Graphics::circle(
                    target,
                    center.0,
                    center.1,
                    radius as u32,
                    fill_color.as_ref(),
                    None,
                );
            } else {
                Graphics::rectangle(target, x, y, width, height, fill_color.as_ref(), None);
            }
        }

        if self.features.contains(UIBoxFeatureFlag::DrawText) {
            let text_content = self.text_content.as_ref().expect("Called UIBox::render() with `UIBoxFeatureFlag::DrawText` when `text_content` is `None`!");

            let text_color = self.styles.text_color.unwrap_or_default();

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

        // Set cursor based on any active drag handle.

        match &self.hot_drag_handle {
            Some(handle) => {
                GLOBAL_UI_CONTEXT.with(|ctx| {
                    *ctx.cursor_kind.borrow_mut() = match handle {
                        UIBoxDragHandle::Left | UIBoxDragHandle::Right => {
                            MouseCursorKind::DragLeftRight
                        }
                        UIBoxDragHandle::Top | UIBoxDragHandle::Bottom => {
                            MouseCursorKind::DragUpDown
                        }
                    };
                });
            }
            None => (),
        }

        Ok(())
    }

    pub fn render_postorder(
        &self,
        children: &[Rc<RefCell<Node<UIBox>>>],
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        if self.features.contains(UIBoxFeatureFlag::DrawChildDividers) && !children.is_empty() {
            let divider_color = self.styles.border_color.unwrap_or_default();

            if children.len() > 1 {
                for i in 0..(children.len() - 1) {
                    let (child_a_rc, child_b_rc) = (&children[i], &children[i + 1]);

                    let (x1, y1, x2, y2) = {
                        let child_a_node = &*child_a_rc.borrow();
                        let child_a_ui_box = &child_a_node.data;

                        let child_b_node = &*child_b_rc.borrow();
                        let child_b_ui_box = &child_b_node.data;

                        let min_top = child_a_ui_box
                            .global_bounds
                            .top
                            .min(child_b_ui_box.global_bounds.top)
                            as i32;

                        let max_bottom = child_a_ui_box
                            .global_bounds
                            .bottom
                            .max(child_b_ui_box.global_bounds.bottom)
                            as i32;

                        let min_left = child_a_ui_box
                            .global_bounds
                            .left
                            .min(child_b_ui_box.global_bounds.left)
                            as i32;

                        let max_right = child_a_ui_box
                            .global_bounds
                            .right
                            .max(child_b_ui_box.global_bounds.right)
                            as i32;

                        match self.layout_direction {
                            UILayoutDirection::TopToBottom => {
                                // Draw a horizontal line across the top of this child.

                                (
                                    min_left,
                                    child_b_ui_box.global_bounds.top as i32,
                                    max_right,
                                    child_b_ui_box.global_bounds.top as i32,
                                )
                            }
                            UILayoutDirection::LeftToRight => {
                                // Draw a vertical line along the left of this child.

                                (
                                    child_b_ui_box.global_bounds.left as i32,
                                    min_top,
                                    child_b_ui_box.global_bounds.left as i32,
                                    max_bottom,
                                )
                            }
                        }
                    };

                    Graphics::line(target, x1, y1, x2, y2, &divider_color);
                }
            }
        }

        #[cfg(debug_assertions)]
        let draw_box_boundaries =
            GLOBAL_UI_CONTEXT.with(|ctx| ctx.debug.borrow().draw_box_boundaries);

        #[cfg(not(debug_assertions))]
        let draw_box_boundaries = false;

        if self.features.contains(UIBoxFeatureFlag::DrawBorder) || draw_box_boundaries {
            let (x, y) = self.get_pixel_coordinates();
            let (width, height) = self.get_computed_pixel_size();

            let (x1, y1, x2, y2) = (
                x as i32,
                y as i32,
                (x + width - 1) as i32,
                (y + height - 1) as i32,
            );

            let border_color = if draw_box_boundaries {
                Some(&color::BLUE)
            } else if self.features.contains(UIBoxFeatureFlag::DrawBorder)
                && self.styles.border_color.is_some()
            {
                self.styles.border_color.as_ref()
            } else {
                None
            };

            let fill_color = if draw_box_boundaries && self.is_spacer() {
                Some(&color::RED)
            } else {
                None
            };

            if self.features.contains(UIBoxFeatureFlag::MaskCircle) {
                let radius = width.min(height) as f32 / 2.0;
                let center = (x + width / 2, y + height / 2);

                Graphics::circle(
                    target,
                    center.0,
                    center.1,
                    radius as u32,
                    fill_color,
                    border_color,
                );
            } else {
                Graphics::rectangle(target, x, y, width, height, fill_color, border_color);
            }

            if self.features.contains(UIBoxFeatureFlag::EmbossAndDeboss) {
                let (mut top_left, mut bottom_right) = (color::WHITE, color::BLACK);

                // Emboss-deboss.

                if self.active {
                    mem::swap(&mut top_left, &mut bottom_right);
                }

                // Top edge.
                target.horizontal_line_unsafe(x1 as u32, x2 as u32, y1 as u32, top_left.to_u32());

                // Bottom edge.
                target.horizontal_line_unsafe(
                    x1 as u32,
                    x2 as u32,
                    y2 as u32,
                    bottom_right.to_u32(),
                );

                // Left edge.
                target.vertical_line_unsafe(x1 as u32, y1 as u32, y2 as u32, top_left.to_u32());

                // Right edge.
                target.vertical_line_unsafe(x2 as u32, y1 as u32, y2 as u32, bottom_right.to_u32());
            }

            // Drag handles

            #[cfg(debug_assertions)]
            {
                if draw_box_boundaries {
                    let handle = match &self.active_drag_handle {
                        Some(active_handle) => Some(active_handle),
                        None => match &self.hot_drag_handle {
                            Some(hot_handle) => Some(hot_handle),
                            None => None,
                        },
                    };

                    let color = match &self.active_drag_handle {
                        Some(_) => color::BLUE.to_u32(),
                        None => match &self.hot_drag_handle {
                            Some(_) => color::RED.to_u32(),
                            None => 0,
                        },
                    };

                    match &handle {
                        Some(handle) => {
                            match handle {
                                UIBoxDragHandle::Top => target
                                    .horizontal_line_unsafe(x1 as u32, x2 as u32, y1 as u32, color),
                                UIBoxDragHandle::Bottom => target
                                    .horizontal_line_unsafe(x1 as u32, x2 as u32, y2 as u32, color),
                                UIBoxDragHandle::Left => target
                                    .vertical_line_unsafe(x1 as u32, y1 as u32, y2 as u32, color),
                                UIBoxDragHandle::Right => target
                                    .vertical_line_unsafe(x2 as u32, y1 as u32, y2 as u32, color),
                            }
                        }
                        None => (),
                    }
                }
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
