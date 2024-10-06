use std::{cell::RefCell, collections::HashMap};

use crate::{
    color::Color,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{cursor::MouseCursorKind, MouseState},
    },
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
};

use super::ui_box::{key::UIKey, styles::UIBoxStylesMap, UIBox};

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

#[derive(Default)]
pub struct UIContext<'a> {
    pub font_cache: RefCell<Option<FontCache<'a>>>,
    pub font_info: RefCell<FontInfo>,
    pub text_cache: RefCell<TextCache>,
    pub styles: RefCell<UIBoxStylesContext>,
    pub global_offset: RefCell<(u32, u32)>,
    pub cache: RefCell<HashMap<UIKey, UIBox>>,
    pub input_events: RefCell<UIInputEvents>,
    pub seconds_since_last_update: RefCell<f32>,
    pub cursor_kind: RefCell<MouseCursorKind>,
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

    pub fn set_user_inputs(
        &self,
        keyboard_state: &mut KeyboardState,
        mouse_state: &mut MouseState,
        game_controller_state: &mut GameControllerState,
    ) {
        let mut input_events = self.input_events.borrow_mut();

        input_events.keyboard = keyboard_state.clone();
        input_events.mouse = mouse_state.clone();
        input_events.game_controller = *game_controller_state;
    }

    pub fn set_seconds_since_last_update(&self, delta_t: f32) {
        let mut seconds_since_last_update = self.seconds_since_last_update.borrow_mut();

        *seconds_since_last_update = delta_t;
    }

    pub fn prune_cache(&self, frame_index: u32) {
        let mut cache = self.cache.borrow_mut();

        cache.retain(|_key, ui_box: &mut UIBox| ui_box.last_read_at_frame == frame_index);
    }
}

thread_local! {
    pub static GLOBAL_UI_CONTEXT: UIContext<'static> = Default::default();
}
