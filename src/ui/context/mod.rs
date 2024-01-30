use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UIID {
    pub parent: u32,
    pub item: u32,
    pub index: u32,
}

impl Display for UIID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UIID {{ parent: {}, item: {}, index: {} }}",
            self.parent, self.item, self.index
        )
    }
}

#[derive(Default, Debug)]
pub struct UIContext {
    hover_target: Option<UIID>,
    focus_target: Option<UIID>,
    is_focus_target_open: bool,
}

impl UIContext {
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

    pub fn is_hovered(&self, id: UIID) -> bool {
        self.hover_target.is_some() && self.hover_target.unwrap() == id
    }

    pub fn is_focused(&self, id: UIID) -> bool {
        self.focus_target.is_some() && self.focus_target.unwrap() == id
    }

    pub fn is_focus_target_open(&self) -> bool {
        self.is_focus_target_open
    }

    pub fn set_focus_target_open(&mut self, is_open: bool) {
        self.is_focus_target_open = is_open;
    }
}
