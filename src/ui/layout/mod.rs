pub mod item;

#[derive(Default, Debug, Copy, Clone)]
pub enum UILayoutDirection {
    LeftToRight,
    #[default]
    TopToBottom,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct UILayoutExtent {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct UILayoutCursor {
    pub x: u32,
    pub y: u32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct UILayoutOptions {
    pub padding: u32,
    pub gap: u32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct UILayoutContext {
    pub direction: UILayoutDirection,
    pub extent: UILayoutExtent,
    cursor: UILayoutCursor,
    pub options: UILayoutOptions,
}

impl UILayoutContext {
    pub fn new(
        direction: UILayoutDirection,
        mut extent: UILayoutExtent,
        options: UILayoutOptions,
    ) -> Self {
        extent.left += options.padding;
        extent.right -= options.padding;
        extent.top += options.padding;
        extent.bottom -= options.padding;

        Self {
            direction,
            extent,
            cursor: UILayoutCursor {
                x: extent.left,
                y: extent.top,
            },
            options,
        }
    }

    pub fn width(&self) -> u32 {
        self.extent.right - self.extent.left
    }

    pub fn height(&self) -> u32 {
        self.extent.bottom - self.extent.top
    }

    pub fn get_cursor(&self) -> &UILayoutCursor {
        &self.cursor
    }

    pub fn advance_cursor(&mut self, item_width: u32, item_height: u32) {
        match self.direction {
            UILayoutDirection::LeftToRight => {
                // Advance the cursor horizontally by the item's width, plus
                // layout spacing.

                self.cursor.x += item_width + self.options.gap;
            }
            UILayoutDirection::TopToBottom => {
                // Advance the cursor vertically by the item's height, plus
                // layout spacing.

                self.cursor.y += item_height + self.options.gap;
            }
        }
    }
}
