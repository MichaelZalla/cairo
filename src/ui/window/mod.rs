use core::fmt;
use std::{cell::RefCell, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{app::resolution::Resolution, color::Color};

use super::{
    context::{UIContext, GLOBAL_UI_CONTEXT},
    panel::tree::PanelTree,
    ui_box::{tree::UIBoxTree, UIBox, UIBoxFeatureFlag, UIBoxFeatureMask, UILayoutDirection},
    UISize, UISizeWithStrictness,
};

pub static DEFAULT_WINDOW_FILL_COLOR: Color = Color::rgb(230, 230, 230);

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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Window<'a, T: Clone + Default + std::fmt::Debug + fmt::Display> {
    pub id: String,
    pub docked: bool,
    pub active: bool,
    pub focused: bool,
    pub position: (u32, u32),
    pub size: (u32, u32),
    #[serde(skip)]
    pub panel_tree: RefCell<PanelTree<'a, T>>,
    #[serde(skip)]
    pub ui_trees: WindowUITrees<'a>,
}

impl<'a, T: Default + Clone + fmt::Debug + Display + Serialize + Deserialize<'a>> Window<'a, T> {
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
                    | UIBoxFeatureFlag::DrawChildDividers,
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
