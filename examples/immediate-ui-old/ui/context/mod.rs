use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
};

use cairo::{
    font::{cache::FontCache, FontInfo},
    graphics::text::cache::TextCache,
};

use super::theme::{UITheme, DEFAULT_UI_THEME};

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UIID {
    pub item: u32,
}

impl Display for UIID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UIID {{ item: {} }}", self.item)
    }
}

#[derive(Debug)]
pub struct UIContext<'a> {
    pub font_cache: &'a mut RefCell<FontCache<'a>>,
    pub font_info: FontInfo,
    pub text_cache: &'a mut RefCell<TextCache>,
    hover_target: Option<UIID>,
    focus_target: Option<UIID>,
    is_focus_target_open: bool,
    theme: Option<&'a UITheme>,
    next_id: u32,
}

impl<'a> UIContext<'a> {
    pub fn new(
        font_cache: &'a mut RefCell<FontCache<'a>>,
        font_info: &FontInfo,
        text_cache: &'a mut RefCell<TextCache>,
    ) -> Self {
        Self {
            font_cache,
            font_info: font_info.clone(),
            text_cache,
            hover_target: None,
            focus_target: None,
            is_focus_target_open: false,
            theme: Some(&DEFAULT_UI_THEME),
            next_id: 0,
        }
    }

    pub fn get_theme(&self) -> &UITheme {
        match &self.theme {
            Some(theme) => theme,
            None => &DEFAULT_UI_THEME,
        }
    }

    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;

        self.next_id += 1;

        id
    }

    pub fn reset_id_counter(&mut self, value: u32) {
        self.next_id = value;
    }

    pub fn get_hover_target(&self) -> Option<UIID> {
        self.hover_target
    }

    pub fn get_focus_target(&self) -> Option<UIID> {
        self.focus_target
    }

    pub fn set_hover_target(&mut self, target: Option<UIID>) {
        self.hover_target = target;
    }

    pub fn set_focus_target(&mut self, target: Option<UIID>) {
        self.focus_target = target;
    }

    pub fn is_hovered(&self, id: &UIID) -> bool {
        self.hover_target.is_some() && self.hover_target.unwrap() == *id
    }

    pub fn is_focused(&self, id: &UIID) -> bool {
        self.focus_target.is_some() && self.focus_target.unwrap() == *id
    }

    pub fn is_focus_target_open(&self) -> bool {
        self.is_focus_target_open
    }

    pub fn set_focus_target_open(&mut self, is_open: bool) {
        self.is_focus_target_open = is_open;
    }
}
