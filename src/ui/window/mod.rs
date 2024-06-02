use core::fmt;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use serde::{Deserialize, Serialize};

use sdl2::mouse::MouseButton;

use crate::{
    app::resolution::Resolution,
    color::{self, Color},
    device::mouse::MouseEventKind,
    mem::linked_list::LinkedList,
};

use super::{
    context::{UIContext, GLOBAL_UI_CONTEXT},
    extent::ScreenExtent,
    panel::tree::PanelTree,
    ui_box::{
        tree::{UIBoxTree, UIBoxTreeRenderCallback},
        utils::{button_box, container_box, text_box},
        UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection,
    },
    UISize, UISizeWithStrictness,
};

pub static DEFAULT_WINDOW_FILL_COLOR: Color = Color::rgb(230, 230, 230);

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
pub struct Window<'a, T: Clone + Default + std::fmt::Debug + fmt::Display> {
    pub id: String,
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
    pub panel_tree: RefCell<PanelTree<'a, T>>,
    #[serde(skip)]
    pub ui_trees: WindowUITrees<'a>,
}

impl<'a, T: Clone + Default + std::fmt::Debug + fmt::Display> fmt::Debug for Window<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("id", &self.id)
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

#[derive(Default, Debug, Copy, Clone)]
pub struct WindowOptions {
    pub docked: bool,
    pub with_titlebar: bool,
    pub position: (u32, u32),
    pub size: (u32, u32),
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

impl<'a, T: Default + Clone + fmt::Debug + Display + Serialize + Deserialize<'a>> Window<'a, T> {
    pub fn new(
        id: String,
        options: WindowOptions,
        render_header_callback: Option<UIBoxTreeRenderCallback>,
        panel_tree: PanelTree<'a, T>,
    ) -> Self {
        Self {
            id,
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

    pub fn render_ui_trees(
        &mut self,
        ctx: &UIContext<'static>,
        main_window_bounds: &Resolution,
    ) -> Result<WindowRenderResult, String> {
        let mut window_render_result = WindowRenderResult {
            did_deactivate: false,
        };

        self.ui_trees.clear();

        let mut render_titlebar_result = None;

        {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            // println!("\nRebuilding tree...\n");

            // Rebuilds the UI tree root based on the current window resolution.

            if self.docked {
                self.size.0 = main_window_bounds.width;
                self.size.1 = main_window_bounds.height;
            }

            let root_ui_box = UIBox::new(
                format!("{}_Root__{}_root", self.id, self.id),
                UIBoxFeatureMask::none()
                    | UIBoxFeatureFlag::DrawFill
                    | UIBoxFeatureFlag::DrawChildDividers
                    | if self.docked {
                        UIBoxFeatureFlag::Null
                    } else {
                        UIBoxFeatureFlag::DrawBorder
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
            );

            {
                *ctx.global_offset.borrow_mut() = self.position;
            }

            ctx.fill_color(DEFAULT_WINDOW_FILL_COLOR, || {
                ui_box_tree.push_parent(root_ui_box)?;

                if self.with_titlebar {
                    render_titlebar_result.replace(render_titlebar(
                        &self.id,
                        self.dragging,
                        ui_box_tree,
                    )?);
                }

                Ok(())
            })?;

            match &self.render_header_callback {
                Some(render) => render(ui_box_tree),
                None => Ok(()),
            }?;
        }

        self.render_panel_tree_to_base_ui_tree()?;

        // Commit this UI tree for the current frame.

        {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            ui_box_tree.commit_frame()?;
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

    fn apply_position_delta(&mut self, delta: (i32, i32), main_window_bounds: &Resolution) {
        let new_x = self.position.0 as i32 + delta.0;
        let new_y = self.position.1 as i32 + delta.1;

        self.position.0 = (new_x.max(0) as u32).min(main_window_bounds.width - self.size.0);
        self.position.1 = (new_y.max(0) as u32).min(main_window_bounds.height - self.size.1);

        self.extent = ScreenExtent::new(self.position, self.size);
    }

    fn render_panel_tree_to_base_ui_tree(&mut self) -> Result<(), String> {
        // Builds UI from the current editor panel tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let panel_tree = &mut self.panel_tree.borrow_mut();

            panel_tree.render(ctx, self)
        })
    }
}

fn render_titlebar(
    id: &str,
    was_dragging: bool,
    tree: &mut UIBoxTree,
) -> Result<WindowRenderTitlebarResult, String> {
    let mut result = WindowRenderTitlebarResult {
        did_start_dragging: false,
        did_stop_dragging: false,
        position_delta: None,
        should_close: false,
    };

    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.fill_color(color::BLACK, || {
            ctx.text_color(color::WHITE, || {
                let container_box_result = tree.push_parent(container_box(
                    format!(
                        "{}_WindowTitleBarContainer__{}_window_titlebar_container",
                        id, id
                    ),
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
                ))?;

                if !was_dragging
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

                tree.push(text_box(
                    format!("{}_WindowTitleBarTitle__{}_window_titlebar_title", id, id),
                    id.to_string(),
                ))?;

                // Spacer

                tree.push(UIBox::new(
                    "".to_string(),
                    UIBoxFeatureMask::none(),
                    UILayoutDirection::LeftToRight,
                    [
                        UISizeWithStrictness {
                            size: UISize::PercentOfParent(1.0),
                            strictness: 0.0,
                        },
                        UISizeWithStrictness {
                            size: UISize::MaxOfSiblings,
                            strictness: 1.0,
                        },
                    ],
                ))?;

                let mut close_button = button_box(
                    format!("{}_WindowTitleBarClose__{}_window_titlebar_close", id, id),
                    "Close".to_string(),
                    None,
                );

                close_button.features ^= UIBoxFeatureFlag::EmbossAndDeboss;

                let close_button_interaction = tree.push(close_button)?;

                if close_button_interaction
                    .mouse_interaction_in_bounds
                    .was_left_pressed
                {
                    result.should_close = true;
                }

                tree.pop_parent()
            })
        })
    })?;

    Ok(result)
}

pub type WindowList<'a, T> = LinkedList<Window<'a, T>>;
