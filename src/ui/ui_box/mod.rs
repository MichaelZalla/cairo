use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use bitflags::bitflags;

use sdl2::mouse::MouseButton;

use crate::{
    animation::{exponential, lerp},
    buffer::Buffer2D,
    collections::tree::node::Node,
    device::mouse::{MouseEventKind, MouseState, cursor::MouseCursorKind},
    resource::handle::Handle,
    ui::context::GLOBAL_UI_CONTEXT,
};

use interaction::UIBoxInteraction;
use key::UIKey;
use styles::UIBoxStyles;
use tree::FocusedTransitionInfo;

use super::{UI_2D_AXIS_COUNT, UISizeWithStrictness, extent::ScreenExtent};

pub mod feature;
pub mod interaction;
pub mod key;
pub mod styles;
pub mod tree;

bitflags! {
    #[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
    pub struct UIBoxFeatureFlags: u32 {
        const DRAW_FILL = 1;
        const DRAW_BORDER = 1 << 1;
        const EMBOSS_AND_DEBOSS = 1 << 2;
        const DRAW_TEXT = 1 << 3;
        const SKIP_TEXT_CACHING = 1 << 4;
        const HOVERABLE = 1 << 5;
        const CLICKABLE = 1 << 6;
        const RESIZABLE_MIN_EXTENT_ON_PRIMARY_AXIS = 1 << 7;
        const RESIZABLE_MAX_EXTENT_ON_PRIMARY_AXIS = 1 << 8;
        const RESIZABLE_MIN_EXTENT_ON_SECONDARY_AXIS = 1 << 9;
        const RESIZABLE_MAX_EXTENT_ON_SECONDARY_AXIS = 1 << 10;
        const DRAW_CHILD_DIVIDERS = 1 << 11;
        const DRAW_CUSTOM_RENDER = 1 << 12;
        const MASK_CIRCLE = 1 << 13;
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
    pub features: UIBoxFeatureFlags,
    pub text_content: Option<String>,
    pub layout_direction: UILayoutDirection,
    pub parent_layout_direction: UILayoutDirection,
    pub semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
    pub styles: UIBoxStyles,
    pub expanded: bool,
    pub selected_item_index: usize,
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
            .field("expanded", &self.expanded)
            .field("selected_item_index", &self.selected_item_index)
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
        mut features: UIBoxFeatureFlags,
        layout_direction: UILayoutDirection,
        semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],
        custom_render_callback: Option<UIBoxCustomRenderCallbackWithContextHandle>,
    ) -> Self {
        let key = UIKey::from(id.clone());

        // Styles may have changed after the previous frame was rendered.

        let (mut fill_color, mut border_color, mut text_color) = (None, None, None);

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let styles = ctx.styles.borrow();

            if let Some(&color) = styles.fill_color.peek() {
                fill_color = Some(color);
            }

            if features.contains(UIBoxFeatureFlags::DRAW_BORDER)
                && let Some(&color) = styles.border_color.peek()
            {
                border_color = Some(color);
            }

            if features.contains(UIBoxFeatureFlags::DRAW_TEXT)
                && let Some(&color) = styles.text_color.peek()
            {
                text_color = Some(color);
            }
        });

        let styles = UIBoxStyles {
            fill_color,
            border_color,
            text_color,
        };

        if custom_render_callback.is_some() {
            features |= UIBoxFeatureFlags::DRAW_CUSTOM_RENDER;
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
        self.hot = if self.features.contains(UIBoxFeatureFlags::HOVERABLE) && !self.key.is_null() {
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

        self.active = if self.features.contains(UIBoxFeatureFlags::CLICKABLE) && !self.key.is_null()
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
        let is_active_transitioning = self.hot_transition < 0.999;
        let is_hot_transitioning = self.hot_transition < 0.999;

        #[cfg(debug_assertions)]
        let draw_active_hover_indicators =
            GLOBAL_UI_CONTEXT.with(|ctx| ctx.debug.borrow().draw_active_hover_indicator);

        #[cfg(not(debug_assertions))]
        let draw_active_hover_indicators = false;

        let should_draw_fill = {
            let has_fill_feature = self.features.contains(UIBoxFeatureFlags::DRAW_FILL);

            if draw_active_hover_indicators {
                has_fill_feature || is_active_transitioning || is_hot_transitioning
            } else {
                has_fill_feature
            }
        };

        // DrawFill path.

        if should_draw_fill {
            self.draw_fill(
                is_hot_transitioning,
                is_active_transitioning,
                draw_active_hover_indicators,
                target,
            );
        }

        // DrawText path.

        if self.features.contains(UIBoxFeatureFlags::DRAW_TEXT) {
            GLOBAL_UI_CONTEXT.with(|ctx| -> Result<(), String> { self.draw_text(ctx, target) })?;
        }

        // Sets cursor based on any active drag handle.

        if let Some(handle) = &self.hot_drag_handle {
            GLOBAL_UI_CONTEXT.with(|ctx| {
                *ctx.cursor_kind.borrow_mut() = match handle {
                    UIBoxDragHandle::Left | UIBoxDragHandle::Right => {
                        MouseCursorKind::DragLeftRight
                    }
                    UIBoxDragHandle::Top | UIBoxDragHandle::Bottom => MouseCursorKind::DragUpDown,
                };
            });
        }

        Ok(())
    }

    pub fn render_postorder(
        &self,
        children: &[Rc<RefCell<Node<UIBox>>>],
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        // DrawChildDividers path.

        if self
            .features
            .contains(UIBoxFeatureFlags::DRAW_CHILD_DIVIDERS)
            && children.len() > 1
        {
            self.draw_child_dividers(children, target);
        }

        // DrawBorder path.

        #[cfg(debug_assertions)]
        let draw_box_boundaries =
            GLOBAL_UI_CONTEXT.with(|ctx| ctx.debug.borrow().draw_box_boundaries);

        #[cfg(not(debug_assertions))]
        let draw_box_boundaries = false;

        if self.features.contains(UIBoxFeatureFlags::DRAW_BORDER) || draw_box_boundaries {
            self.draw_border(draw_box_boundaries, target);
        }

        // EmbossAndDeboss path.

        if self.features.contains(UIBoxFeatureFlags::EMBOSS_AND_DEBOSS) {
            self.emboss_and_deboss(target);
        }

        // DrawDebugDragHandles path.

        #[cfg(debug_assertions)]
        let draw_drag_handles = GLOBAL_UI_CONTEXT.with(|ctx| ctx.debug.borrow().draw_drag_handles);

        #[cfg(debug_assertions)]
        if draw_drag_handles {
            self.draw_debug_drag_handles(target);
        }

        Ok(())
    }

    pub(self) fn cache(&self, cache: &mut HashMap<UIKey, UIBox>, frame_index: u32) {
        if cache.contains_key(&self.key) {
            let cached = cache.get_mut(&self.key).unwrap();

            cached.global_bounds = self.global_bounds;
            cached.expanded = self.expanded;
            cached.selected_item_index = self.selected_item_index;
            cached.hot = self.hot;
            cached.hot_transition = self.hot_transition;
            cached.active = self.active;
            cached.active_transition = self.active_transition;
            cached.focused = self.focused;
            cached.hot_drag_handle = self.hot_drag_handle;
            cached.active_drag_handle = self.active_drag_handle;
            cached.last_read_at_frame = frame_index;
        } else if !self.key.is_null() {
            cache.insert(self.key.clone(), self.clone());
        }
    }
}

impl fmt::Display for UIBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UIBox(id=\"{}\", hash={})", self.id, self.key)
    }
}
