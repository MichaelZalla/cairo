use crate::{buffer::Buffer2D, color::Color};

use super::Graphics;

impl Graphics {
    pub fn rectangle(
        buffer: &mut Buffer2D,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        fill: Option<&Color>,
        border: Option<&Color>,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        let y_start = y.min(buffer.height - 1);
        let y_end_inclusive = (y + height - 1).min(buffer.height - 1);

        let x_start = x.min(buffer.width - 1);
        let x_end_inclusive = (x + width - 1).min(buffer.width - 1);

        if y_start == y_end_inclusive || x_start == x_end_inclusive {
            return;
        }

        // Render a fill.

        if let Some(fill_color) = fill {
            for current_y in y_start..y_end_inclusive + 1 {
                buffer.horizontal_line_unsafe(
                    x_start,
                    x_end_inclusive,
                    current_y,
                    fill_color.to_u32(),
                )
            }
        }

        // Render a border.

        if let Some(border_color) = border {
            let color_u32 = border_color.to_u32();

            if y_start == y {
                // Top edge was not clipped.
                buffer.horizontal_line_unsafe(x_start, x_end_inclusive, y_start, color_u32);
            }

            if y_end_inclusive == y + height - 1 {
                // Bottom edge was not clipped.
                buffer.horizontal_line_unsafe(x_start, x_end_inclusive, y_end_inclusive, color_u32);
            }

            if x_start == x {
                // Left edge was not clipped.
                buffer.vertical_line_unsafe(x_start, y_start, y_end_inclusive, color_u32);
            }

            if x_end_inclusive == x + width - 1 {
                // Right edge was not clipped.
                buffer.vertical_line_unsafe(x_end_inclusive, y_start, y_end_inclusive, color_u32);
            }
        }
    }

    pub fn clip_rectangle(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        buffer: &Buffer2D,
    ) -> Option<(u32, u32, u32, u32)> {
        // Overlapping tests.

        if x >= buffer.width as i32 || (x + width as i32) < 0 {
            return None;
        }

        if y >= buffer.height as i32 || (y + height as i32) < 0 {
            return None;
        }

        let (left, top, right, bottom) = (
            x.max(0).min(buffer.width as i32 - 1),
            y.max(0).min(buffer.height as i32 - 1),
            (x + width as i32).max(0).min(buffer.width as i32 - 1),
            (y + height as i32).max(0).min(buffer.height as i32 - 1),
        );

        Some((
            left as u32,
            top as u32,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
    }
}
