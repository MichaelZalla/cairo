use crate::{buffer::Buffer2D, color, graphics::Graphics};

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
    current_row_height: u32,
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
            current_row_height: 0,
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

    pub fn get_current_row_height(&self) -> u32 {
        self.current_row_height
    }

    pub fn prepare_cursor(&mut self, item_width: u32, _item_height: u32) {
        match self.direction {
            UILayoutDirection::LeftToRight => {
                // Begin a new row, if the requested draw size won't fit on our
                // current row.
                let remaining_layout_width =
                    (self.extent.left + self.width()) as i32 - self.get_cursor().x as i32;

                if item_width as i32 > remaining_layout_width {
                    self.cursor.x = self.options.padding;
                    self.cursor.y += self.current_row_height;

                    self.current_row_height = 0;
                }
            }
            UILayoutDirection::TopToBottom => {
                // Do nothing.
            }
        }
    }

    pub fn advance_cursor(&mut self, item_width: u32, item_height: u32) {
        match self.direction {
            UILayoutDirection::LeftToRight => {
                // Advance the cursor horizontally by the item's width, plus
                // layout spacing.

                self.cursor.x += item_width + self.options.gap;

                self.current_row_height = self.current_row_height.max(item_height);

                self.prepare_cursor(item_width, item_height);
            }
            UILayoutDirection::TopToBottom => {
                // Advance the cursor vertically by the item's height, plus
                // layout spacing.

                self.cursor.y += item_height + self.options.gap;
            }
        }
    }

    pub fn draw_debug_bounds(&self, parent_buffer: &mut Buffer2D) {
        // Outer bounds (outside padding)

        Graphics::rectangle(
            parent_buffer,
            self.extent.left - self.options.padding,
            self.extent.top - self.options.padding,
            self.width() + self.options.padding * 2,
            self.height() + self.options.padding * 2,
            None,
            Some(color::GREEN),
        );

        // Inner bounds (inside padding)

        Graphics::rectangle(
            parent_buffer,
            self.extent.left,
            self.extent.top,
            self.width(),
            self.height(),
            None,
            Some(color::YELLOW),
        );
    }
}
