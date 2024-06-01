use std::cell::RefCell;

use serde::{Deserialize, Serialize};

use super::{panel::tree::PanelTree, ui_box::tree::UIBoxTree};

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
pub struct Window<'a, T: Default> {
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

pub type WindowList<'a, T> = Vec<Window<'a, T>>;
