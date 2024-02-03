use std::sync::RwLockWriteGuard;

use crate::{buffer::Buffer2D, color::Color, graphics::Graphics};

use super::{
    context::{UIContext, UIID},
    layout::{UILayoutContext, UILayoutDirection},
    theme::DEFAULT_UI_THEME,
};

static SEPARATOR_MARGIN: u32 = 8;

#[derive(Debug)]
pub struct SeparatorOptions {
    pub color: Color,
}

impl Default for SeparatorOptions {
    fn default() -> Self {
        Self {
            color: DEFAULT_UI_THEME.separator,
        }
    }
}

#[derive(Default, Debug)]
pub struct DoSeparatorResult {}

pub fn do_separator(
    _ctx: &mut RwLockWriteGuard<'_, UIContext>,
    _id: UIID,
    layout: &mut UILayoutContext,
    options: &SeparatorOptions,
    parent_buffer: &mut Buffer2D,
) -> DoSeparatorResult {
    let item_width = match layout.direction {
        UILayoutDirection::LeftToRight => 1 + SEPARATOR_MARGIN * 2,
        UILayoutDirection::TopToBottom => layout.width(),
    };
    let item_height = match layout.direction {
        UILayoutDirection::LeftToRight => layout.get_current_row_height(),
        UILayoutDirection::TopToBottom => 1 + SEPARATOR_MARGIN * 2,
    };

    layout.prepare_cursor(item_width, item_height);

    draw_separator(layout, options, parent_buffer);

    layout.advance_cursor(item_width, item_height);

    DoSeparatorResult {}
}

fn draw_separator(
    layout: &UILayoutContext,
    options: &SeparatorOptions,
    parent_buffer: &mut Buffer2D,
) {
    let cursor = layout.get_cursor();

    let color = options.color;

    // Draw the separator (with margins).

    match layout.direction {
        UILayoutDirection::LeftToRight => {
            // Draw a vertical separator.

            Graphics::line(
                parent_buffer,
                (cursor.x + SEPARATOR_MARGIN) as i32,
                cursor.y as i32,
                (cursor.x + SEPARATOR_MARGIN) as i32,
                (cursor.y + layout.get_current_row_height()) as i32,
                color,
            );
        }
        UILayoutDirection::TopToBottom => {
            // Draw a horizontal separator.

            Graphics::line(
                parent_buffer,
                layout.extent.left as i32,
                (cursor.y + SEPARATOR_MARGIN) as i32,
                layout.width() as i32,
                (cursor.y + SEPARATOR_MARGIN) as i32,
                color,
            );
        }
    }
}
