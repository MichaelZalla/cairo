use std::{cell::RefCell, fmt, rc::Rc};

use serde::{Deserialize, Serialize};

use sdl2::mouse::MouseButton;

use crate::{
    app::resolution::Resolution,
    buffer::Buffer2D,
    device::mouse::{MouseEventKind, MouseState},
};

use super::{
    context::{UIContext, GLOBAL_UI_CONTEXT},
    extent::ScreenExtent,
    panel::tree::PanelTree,
    ui_box::{
        tree::{UIBoxTree, UIBoxTreeRenderCallback},
        utils::{button, container, greedy_spacer, text},
        UIBox, UIBoxDragHandle, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

pub mod list;

pub type WindowRenderCallback = Rc<dyn Fn(&mut UIBoxTree) -> Result<(), String>>;

#[derive(Default, Debug, Clone)]
pub struct WindowUITrees<'a> {
    pub base: RefCell<UIBoxTree<'a>>,
    pub dropdowns: RefCell<UIBoxTree<'a>>,
    pub tooltips: RefCell<UIBoxTree<'a>>,
}

impl<'a> WindowUITrees<'a> {
    pub fn clear(&self) {
        self.base.borrow_mut().clear();
        self.dropdowns.borrow_mut().clear();
        self.tooltips.borrow_mut().clear();
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Window<'a> {
    pub id: String,
    pub title: String,
    pub docked: bool,
    pub dragging: bool,
    pub active: bool,
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub extent: ScreenExtent,
    pub with_titlebar: bool,
    #[serde(skip)]
    pub render_header_callback: Option<UIBoxTreeRenderCallback>,
    #[serde(skip)]
    pub panel_tree: RefCell<PanelTree<'a>>,
    #[serde(skip)]
    pub ui_trees: WindowUITrees<'a>,
}

impl<'a> fmt::Debug for Window<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("docked", &self.docked)
            .field("dragging", &self.dragging)
            .field("active", &self.active)
            .field("position", &self.position)
            .field("size", &self.size)
            .field("extent", &self.extent)
            .field("with_titlebar", &self.with_titlebar)
            .field(
                "render_callback",
                match self.render_header_callback {
                    Some(_) => &"Some(Rc<dyn Fn(&mut UIBoxTree) -> Result<(), String>>)",
                    None => &"None ",
                },
            )
            .field("panel_tree", &self.panel_tree)
            .field("ui_trees", &self.ui_trees)
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WindowOptions {
    pub docked: bool,
    pub with_titlebar: bool,
    pub position: (u32, u32),
    pub size: (u32, u32),
}

impl WindowOptions {
    pub fn docked(resolution: Resolution) -> Self {
        Self {
            docked: true,
            with_titlebar: false,
            position: (0, 0),
            size: (resolution.width, resolution.height),
        }
    }
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            docked: false,
            with_titlebar: true,
            position: (8, 8),
            size: (200, 200),
        }
    }
}

pub struct WindowRenderResult {
    pub did_deactivate: bool,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct WindowRenderTitlebarResult {
    pub did_start_dragging: bool,
    pub did_stop_dragging: bool,
    pub position_delta: Option<(i32, i32)>,
    pub should_close: bool,
}

impl<'a> Window<'a> {
    pub fn new(
        id: String,
        title: String,
        options: WindowOptions,
        render_header_callback: Option<UIBoxTreeRenderCallback>,
        panel_tree: PanelTree<'a>,
    ) -> Self {
        Self {
            id,
            title,
            docked: options.docked,
            active: true,
            with_titlebar: options.with_titlebar,
            position: options.position,
            size: options.size,
            extent: ScreenExtent::new(options.position, options.size),
            render_header_callback,
            panel_tree: RefCell::new(panel_tree),
            ..Default::default()
        }
    }

    pub fn rebuild_ui_trees(
        &mut self,
        ctx: &UIContext<'static>,
        main_window_bounds: &Resolution,
    ) -> Result<WindowRenderResult, String> {
        let mut window_render_result = WindowRenderResult {
            did_deactivate: false,
        };

        self.ui_trees.clear();

        let mut render_titlebar_result = None;

        // Rebuilds the UI tree root based on the current window resolution.

        if self.docked {
            self.size.0 = main_window_bounds.width;
            self.size.1 = main_window_bounds.height;
        }

        let mut root_ui_box: UIBox = Default::default();

        let theme = ctx.theme.borrow();

        ctx.fill_color(theme.panel_background, || {
            ctx.border_color(theme.panel_border, || {
                root_ui_box = UIBox::new(
                    format!("{}_Root__{}_root", self.id, self.id),
                    UIBoxFeatureMask::none()
                        | UIBoxFeatureFlag::DrawFill
                        | UIBoxFeatureFlag::DrawChildDividers
                        | if self.docked {
                            UIBoxFeatureMask::none()
                        } else {
                            UIBoxFeatureFlag::DrawBorder
                                | UIBoxFeatureFlag::ResizableMinExtentOnPrimaryAxis
                                | UIBoxFeatureFlag::ResizableMaxExtentOnPrimaryAxis
                                | UIBoxFeatureFlag::ResizableMinExtentOnSecondaryAxis
                                | UIBoxFeatureFlag::ResizableMaxExtentOnSecondaryAxis
                        },
                    UILayoutDirection::TopToBottom,
                    [
                        UISizeWithStrictness {
                            size: UISize::Pixels(self.size.0),
                            strictness: 1.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::Pixels(self.size.1),
                            strictness: 1.0,
                        },
                    ],
                    None,
                );

                Ok(())
            })?;

            Ok(())
        })?;

        {
            *ctx.global_offset.borrow_mut() = self.position;
        }

        let root_ui_box_result;

        {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            root_ui_box_result = ui_box_tree.push_parent(root_ui_box)?;
        }

        if self.with_titlebar {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            render_titlebar_result.replace(render_titlebar(
                &self.id,
                &self.title,
                self.dragging,
                &root_ui_box_result
                    .mouse_interaction_in_bounds
                    .active_drag_handle,
                ui_box_tree,
            )?);

            match &self.render_header_callback {
                Some(render) => render(ui_box_tree),
                None => Ok(()),
            }?;
        }

        {
            // Builds UI from the current editor panel tree.

            let panel_tree = &mut self.panel_tree.borrow_mut();

            panel_tree.render(self)?;
        }

        // Commit this UI tree for the current frame.

        {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            ui_box_tree.commit_frame()?;
        }

        match &root_ui_box_result
            .mouse_interaction_in_bounds
            .active_drag_handle
        {
            Some(handle) => {
                let mouse = &ctx.input_events.borrow().mouse;

                if mouse.drag_event.is_some() {
                    self.apply_resize_event(mouse, handle, main_window_bounds);
                }
            }
            None => (),
        }

        {
            *ctx.global_offset.borrow_mut() = (0, 0);
        }

        if let Some(result) = render_titlebar_result {
            if result.did_start_dragging {
                self.dragging = true;
            }

            if result.did_stop_dragging {
                self.dragging = false;
            }

            if self.dragging {
                if let Some(delta) = result.position_delta {
                    self.apply_position_delta(delta, main_window_bounds);
                }
            }

            if result.should_close {
                self.active = false;

                window_render_result.did_deactivate = true;
            }
        }

        Ok(window_render_result)
    }

    pub fn render(&self, frame_index: u32, framebuffer: &mut Buffer2D) -> Result<(), String> {
        let base_ui_tree = &mut self.ui_trees.base.borrow_mut();

        // Render the window's base UI tree into the framebuffer for the current frame.

        base_ui_tree.render_frame(frame_index, framebuffer)
    }

    fn get_computed_size_of_root_child(&self, index: usize) -> Option<(u32, u32)> {
        let base_ui_tree = self.ui_trees.base.borrow();
        let root_rc_option = &base_ui_tree.tree.root;

        if let Some(root_rc) = root_rc_option {
            let root = root_rc.as_ref().borrow();
            let titlebar_root = &root.children[index].borrow().data;

            GLOBAL_UI_CONTEXT.with(|ctx| {
                ctx.cache
                    .borrow()
                    .get(&titlebar_root.key)
                    .map(|cached_ui_box| cached_ui_box.get_computed_pixel_size())
            })
        } else {
            None
        }
    }

    fn get_computed_size_of_titlebar(&self) -> Option<(u32, u32)> {
        self.get_computed_size_of_root_child(0)
    }

    fn get_computed_size_of_panel_tree(&self) -> Option<(u32, u32)> {
        self.get_computed_size_of_root_child(1)
    }

    fn get_computed_size(&self) -> (u32, u32) {
        // Computes the minimum extent needed to fit the titlebar (height) and panel tree content.
        // Note: The titlebar's width is always equal to this window's width.

        let titlebar_computed_size = self.get_computed_size_of_titlebar().unwrap_or_default();
        let panel_tree_computed_size = self.get_computed_size_of_panel_tree().unwrap_or_default();

        (
            panel_tree_computed_size.0,
            titlebar_computed_size.1 + panel_tree_computed_size.1,
        )
    }

    fn apply_position_delta(&mut self, delta: (i32, i32), main_window_bounds: &Resolution) {
        let new_x = self.position.0 as i32 + delta.0;
        let new_y = self.position.1 as i32 + delta.1;

        self.position.0 = (new_x.max(0) as u32).min(main_window_bounds.width - self.size.0);
        self.position.1 = (new_y.max(0) as u32).min(main_window_bounds.height - self.size.1);

        self.extent = ScreenExtent::new(self.position, self.size);
    }

    fn apply_resize_event(
        &mut self,
        mouse_state: &MouseState,
        handle: &UIBoxDragHandle,
        main_window_bounds: &Resolution,
    ) {
        let mut delta = mouse_state.drag_event.as_ref().unwrap().delta;

        // Restricts the resize deltas to protect the minimum size needed for
        // the window's inner content.

        let minimum_size = self.get_computed_size();

        let buffer = (
            self.size.0.max(minimum_size.0) as i32 - minimum_size.0 as i32,
            self.size.1.max(minimum_size.1) as i32 - minimum_size.1 as i32,
        );

        match &handle {
            UIBoxDragHandle::Left => {
                if delta.0 > 0 {
                    delta.0 = delta.0.min(buffer.0);
                } else {
                    delta.0 = delta.0.max(-(self.extent.left as i32));
                }

                self.size.0 = (self.size.0 as i32 - delta.0) as u32;

                self.apply_position_delta((delta.0, 0), main_window_bounds);
            }
            UIBoxDragHandle::Right => {
                if delta.0 < 0 {
                    delta.0 = delta.0.max(-buffer.0);
                } else {
                    delta.0 = delta
                        .0
                        .min(main_window_bounds.width as i32 - 1 - self.extent.right as i32);
                }

                self.size.0 = (self.size.0 as i32 + delta.0) as u32;
            }
            UIBoxDragHandle::Top => {
                if delta.1 > 0 {
                    delta.1 = delta.1.min(buffer.1);
                } else {
                    delta.1 = delta.1.max(-(self.extent.top as i32));
                }

                self.size.1 = (self.size.1 as i32 - delta.1) as u32;

                self.apply_position_delta((0, delta.1), main_window_bounds);
            }
            UIBoxDragHandle::Bottom => {
                if delta.1 < 0 {
                    delta.1 = delta.1.max(-buffer.1);
                } else {
                    delta.1 = delta
                        .1
                        .min(main_window_bounds.height as i32 - 1 - self.extent.bottom as i32);
                }

                self.size.1 = (self.size.1 as i32 + delta.1) as u32;
            }
        }

        self.extent = ScreenExtent::new(self.position, self.size);
    }
}

fn render_titlebar(
    id: &str,
    title: &str,
    was_dragging: bool,
    active_drag_handle: &Option<UIBoxDragHandle>,
    tree: &mut UIBoxTree,
) -> Result<WindowRenderTitlebarResult, String> {
    let mut result = WindowRenderTitlebarResult {
        did_start_dragging: false,
        did_stop_dragging: false,
        position_delta: None,
        should_close: false,
    };

    GLOBAL_UI_CONTEXT.with(|ctx| {
        let theme = ctx.theme.borrow();

        ctx.fill_color(theme.panel_titlebar_background, || {
            let titlebar_container = container(
                format!("{}_WindowTitleBarContainer", id),
                UILayoutDirection::LeftToRight,
                Some([
                    UISizeWithStrictness {
                        size: UISize::ChildrenSum,
                        strictness: 1.0,
                    },
                    UISizeWithStrictness {
                        size: UISize::PercentOfParent(1.0),
                        strictness: 1.0,
                    },
                ]),
            );

            let container_box_result = tree.push_parent(titlebar_container)?;

            // @TODO Generalize this over all UIBox instances with Draggable
            // feature enabled.

            if active_drag_handle.is_none()
                && !was_dragging
                && container_box_result
                    .mouse_interaction_in_bounds
                    .was_left_pressed
            {
                result.did_start_dragging = true;
            } else if was_dragging {
                GLOBAL_UI_CONTEXT.with(|ctx| {
                    let mouse_state = &ctx.input_events.borrow().mouse;

                    let did_stop_dragging = if let Some(event) = mouse_state.button_event {
                        matches!(
                            (event.button, event.kind),
                            (MouseButton::Left, MouseEventKind::Up)
                        )
                    } else {
                        false
                    };

                    result.did_stop_dragging = did_stop_dragging;

                    if !did_stop_dragging {
                        result.position_delta = Some(mouse_state.relative_motion);
                    }
                });
            }

            tree.push(text(
                format!("{}_WindowTitleBarTitle", id),
                title.to_string(),
            ))?;

            // Spacer

            tree.push(greedy_spacer())?;

            let mut close_button = button(
                format!("{}_WindowTitleBarClose", id),
                "Close".to_string(),
                None,
            );

            close_button.features ^= UIBoxFeatureFlag::DrawFill | UIBoxFeatureFlag::EmbossAndDeboss;

            let close_button_interaction = tree.push(close_button)?;

            if close_button_interaction
                .mouse_interaction_in_bounds
                .was_left_pressed
            {
                result.should_close = true;
            }

            tree.pop_parent()
        })
    })?;

    Ok(result)
}
