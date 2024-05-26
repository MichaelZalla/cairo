use std::{cell::RefCell, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::color::Color;

use super::{
    tree::UIBoxTree,
    ui_box::{key::UIKey, UIBox},
};

#[derive(Default, Debug, Clone)]
pub struct UIBoxStyleStack<T> {
    stack: Vec<T>,
}

impl<T> UIBoxStyleStack<T> {
    pub fn push(&mut self, item: T) {
        self.stack.push(item)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.stack.last()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIBoxStylesMap<T: Default + Clone> {
    pub fill_color: T,
    pub border_color: T,
}

pub type UIBoxStyles = UIBoxStylesMap<Option<Color>>;
pub type UIBoxStylesContext = UIBoxStylesMap<UIBoxStyleStack<Color>>;

#[derive(Default, Debug, Clone)]
pub struct UIContext<'a> {
    pub styles: RefCell<UIBoxStylesContext>,
    pub tree: RefCell<UIBoxTree<'a>>,
    pub dropdown_menus: RefCell<UIBoxTree<'a>>,
    pub tooltips: RefCell<UIBoxTree<'a>>,
    pub cache: RefCell<HashMap<UIKey, UIBox>>,
}

macro_rules! with_style {
    ($style: ident) => {
        pub fn $style<C>(&self, $style: Color, callback: C) -> Result<(), String>
        where
            C: FnOnce() -> Result<(), String>,
        {
            {
                let mut styles = self.styles.borrow_mut();

                styles.$style.push($style);
            }

            let result = callback();

            {
                let mut styles = self.styles.borrow_mut();

                styles.$style.pop();
            }

            result
        }
    };
}

impl<'a> UIContext<'a> {
    with_style!(fill_color);
    with_style!(border_color);
}

thread_local! {
    pub static GLOBAL_UI_CONTEXT: UIContext<'static> = Default::default();
}
