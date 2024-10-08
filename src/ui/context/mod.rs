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

use theme::UITheme;

pub mod theme;

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

#[cfg(debug_assertions)]
#[derive(Default, Debug, Copy, Clone)]
pub struct UIContextDebugOptions {
    pub draw_box_boundaries: bool,
}

pub struct UIContext<'a> {
    pub font_cache: RefCell<Option<FontCache<'a>>>,
    pub font_info: RefCell<FontInfo>,
    pub text_cache: RefCell<TextCache>,
    pub theme: RefCell<UITheme>,
    pub styles: RefCell<UIBoxStylesContext>,
    pub global_offset: RefCell<(u32, u32)>,
    pub cache: RefCell<HashMap<UIKey, UIBox>>,
    pub input_events: RefCell<UIInputEvents>,
    pub seconds_since_last_update: RefCell<f32>,
    pub cursor_kind: RefCell<MouseCursorKind>,
    #[cfg(debug_assertions)]
    pub debug: RefCell<UIContextDebugOptions>,
}

impl<'a> Default for UIContext<'a> {
    fn default() -> Self {
        let default_theme = UITheme::default();

        let styles = UIBoxStylesMap::<UIBoxStyleStack<Color>> {
            fill_color: UIBoxStyleStack::<Color> { stack: vec![] },
            border_color: UIBoxStyleStack::<Color> { stack: vec![] },
            text_color: UIBoxStyleStack::<Color> {
                stack: vec![default_theme.text],
            },
        };

        Self {
            font_cache: Default::default(),
            font_info: Default::default(),
            text_cache: Default::default(),
            theme: RefCell::new(default_theme),
            styles: RefCell::new(styles),
            global_offset: Default::default(),
            cache: Default::default(),
            input_events: Default::default(),
            seconds_since_last_update: Default::default(),
            cursor_kind: Default::default(),
            #[cfg(debug_assertions)]
            debug: Default::default(),
        }
    }
}

macro_rules! with_style_applied {
    ($style: ident) => {
        pub fn $style<C, T>(&self, $style: Color, callback: C) -> Result<T, String>
        where
            C: FnOnce() -> Result<T, String>,
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

    pub fn begin_frame(&self) {
        *self.cursor_kind.borrow_mut() = MouseCursorKind::Arrow;
    }

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
