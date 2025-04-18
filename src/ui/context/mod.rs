use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    app::App,
    color::Color,
    device::{
        game_controller::GameControllerState,
        keyboard::KeyboardState,
        mouse::{cursor::MouseCursorKind, MouseState},
    },
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
    resource::arena::Arena,
    texture::map::TextureMap,
    time::TimingInfo,
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
    pub draw_active_hover_indicator: bool,
    pub draw_drag_handles: bool,
}

pub struct UIContext {
    pub font_cache: RefCell<Option<FontCache>>,
    pub font_info: RefCell<FontInfo>,
    pub text_cache: RefCell<TextCache>,
    pub image_arena: RefCell<Option<Rc<RefCell<Arena<TextureMap>>>>>,
    pub theme: RefCell<UITheme>,
    pub styles: RefCell<UIBoxStylesContext>,
    pub global_offset: RefCell<(u32, u32)>,
    pub cache: RefCell<HashMap<UIKey, UIBox>>,
    pub input_events: RefCell<UIInputEvents>,
    pub timing_info: RefCell<TimingInfo>,
    pub cursor_kind: RefCell<MouseCursorKind>,
    #[cfg(debug_assertions)]
    pub debug: RefCell<UIContextDebugOptions>,
}

impl Default for UIContext {
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
            image_arena: Default::default(),
            theme: RefCell::new(default_theme),
            styles: RefCell::new(styles),
            global_offset: Default::default(),
            cache: Default::default(),
            input_events: Default::default(),
            timing_info: Default::default(),
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

impl UIContext {
    with_style_applied!(fill_color);
    with_style_applied!(border_color);
    with_style_applied!(text_color);

    pub fn load_font(&self, app: &App, font_path: String, point_size: u16) {
        self.font_cache
            .borrow_mut()
            .replace(FontCache::new(app.context.ttf_context));

        {
            let mut font_info = self.font_info.borrow_mut();

            font_info.filepath = font_path;
            font_info.point_size = point_size;
        }
    }

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

    pub fn set_timing_info(&self, timing_info: &TimingInfo) {
        *self.timing_info.borrow_mut() = *timing_info;
    }

    pub fn prune_cache(&self, frame_index: u32) {
        let mut cache = self.cache.borrow_mut();

        cache.retain(|_key, ui_box: &mut UIBox| ui_box.last_read_at_frame == frame_index);
    }
}

thread_local! {
    pub static GLOBAL_UI_CONTEXT: UIContext = Default::default();
}
