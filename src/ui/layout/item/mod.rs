use super::UILayoutContext;

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
    pub fn get_layout_offset(&self, layout: &UILayoutContext, item_width: u32) -> (u32, u32) {
        let remaining_layout_width = layout.width() - layout.get_cursor().x;

        let x = match self.horizontal_alignment {
            ItemLayoutHorizontalAlignment::Left => self.x_offset,
            ItemLayoutHorizontalAlignment::Center => {
                ((remaining_layout_width as f32 / 2.0 - item_width as f32 / 2.0)
                    - (self.x_offset as f32 / 2.0)) as u32
            }
            ItemLayoutHorizontalAlignment::Right => {
                remaining_layout_width - item_width - self.x_offset
            }
        };

        let y = self.y_offset;

        (x, y)
    }
}
