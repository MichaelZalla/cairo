#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UIID {
    pub parent: u32,
    pub item: u32,
    pub index: u32,
}

#[derive(Default, Debug)]
pub struct UIContext {
    hover_target: Option<UIID>,
    focus_target: Option<UIID>,
}

impl UIContext {
    pub fn get_hover_target(&self) -> Option<UIID> {
        self.hover_target
    }

    pub fn get_focus_target(&self) -> Option<UIID> {
        self.focus_target
    }

    pub fn set_hover_target(&mut self, target: Option<UIID>) {
        // println!("Setting self.hover_target to {:#?}.", target);

        self.hover_target = target;
    }

    pub fn set_focus_target(&mut self, target: Option<UIID>) {
        // println!("Setting self.focus_target to {:#?}.", target);

        self.focus_target = target;
    }
}
