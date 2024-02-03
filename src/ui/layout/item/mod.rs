use crate::ui::panel::PanelInfo;

#[derive(Default, Copy, Clone, Debug)]
pub enum ItemLayoutHorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Copy, Clone, Debug)]
pub enum ItemTextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Debug)]
pub struct ItemLayoutOptions {
    pub x_offset: u32,
    pub y_offset: u32,
    pub horizontal_alignment: ItemLayoutHorizontalAlignment,
}

impl ItemLayoutOptions {
    pub fn get_top_left_within_parent(&self, parent: &PanelInfo, width: u32) -> (u32, u32) {
        let x = match self.horizontal_alignment {
            ItemLayoutHorizontalAlignment::Left => self.x_offset,
            ItemLayoutHorizontalAlignment::Center => {
                ((parent.width as f32 / 2.0 - width as f32 / 2.0) - (self.x_offset as f32 / 2.0))
                    as u32
            }
            ItemLayoutHorizontalAlignment::Right => parent.width - width - self.x_offset,
        };

        let y = self.y_offset;

        (x, y)
    }
}
