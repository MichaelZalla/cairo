use core::fmt;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{app::resolution::Resolution, color::Color};

use super::{
    context::{UIContext, GLOBAL_UI_CONTEXT},
    extent::ScreenExtent,
    panel::tree::PanelTree,
    ui_box::{
        tree::{UIBoxTree, UIBoxTreeRenderCallback},
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
    pub active: bool,
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub extent: ScreenExtent,
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
            .field("active", &self.active)
            .field("position", &self.position)
            .field("size", &self.size)
            .field("extent", &self.extent)
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
    pub position: (u32, u32),
    pub size: (u32, u32),
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
        resolution: &Resolution,
    ) -> Result<(), String> {
        self.ui_trees.clear();

        {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            // println!("\nRebuilding tree...\n");

            // Rebuilds the UI tree root based on the current window resolution.

            if self.docked {
                self.size.0 = resolution.width;
                self.size.1 = resolution.height;
            }

            let root_ui_box = UIBox::new(
                "Root__root".to_string(),
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

            ctx.fill_color(DEFAULT_WINDOW_FILL_COLOR, || {
                ui_box_tree.push_parent(root_ui_box)?;

                Ok(())
            })?;

            match &self.render_header_callback {
                Some(render) => render(ui_box_tree),
                None => Ok(()),
            }?;
        }

        self.render_panel_tree_to_base_ui_tree()
    }

    fn render_panel_tree_to_base_ui_tree(&mut self) -> Result<(), String> {
        // Builds UI from the current editor panel tree.

        GLOBAL_UI_CONTEXT.with(|ctx| {
            let panel_tree = &mut self.panel_tree.borrow_mut();

            panel_tree.render(ctx, self).unwrap();
        });

        // Commit this UI tree for the current frame.

        {
            let ui_box_tree = &mut self.ui_trees.base.borrow_mut();

            ui_box_tree.commit_frame()?;
        }

        Ok(())
    }
}

pub type WindowList<'a, T> = Vec<Window<'a, T>>;
