use std::{cell::RefCell, collections::HashMap};

use crate::{
    color::Color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
};

use super::ui_box::{key::UIKey, styles::UIBoxStylesMap, tree::UIBoxTree, UIBox};

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

pub type UIBoxStylesContext = UIBoxStylesMap<UIBoxStyleStack<Color>>;

#[derive(Default, Debug, Clone)]
pub struct UIInputEvents {
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
    pub game_controller: GameControllerState,
}

#[derive(Default, Debug)]
pub struct UIContext<'a> {
    pub font_cache: RefCell<Option<FontCache<'a>>>,
    pub font_info: RefCell<FontInfo>,
    pub text_cache: RefCell<TextCache>,
    pub styles: RefCell<UIBoxStylesContext>,
    pub tree: RefCell<UIBoxTree<'a>>,
    pub dropdown_menus: RefCell<UIBoxTree<'a>>,
    pub tooltips: RefCell<UIBoxTree<'a>>,
    pub cache: RefCell<HashMap<UIKey, UIBox>>,
    pub input_events: RefCell<UIInputEvents>,
    pub seconds_since_last_update: RefCell<f32>,
}

macro_rules! with_style_applied {
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
    with_style_applied!(fill_color);
    with_style_applied!(border_color);
    with_style_applied!(text_color);

    pub fn clear_for_next_frame(&self) {
        self.tree.borrow_mut().clear();
        self.dropdown_menus.borrow_mut().clear();
        self.tooltips.borrow_mut().clear();
    }
}

thread_local! {
    pub static GLOBAL_UI_CONTEXT: UIContext<'static> = Default::default();
}
