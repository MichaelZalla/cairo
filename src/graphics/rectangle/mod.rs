use std::fmt::Debug;

use crate::{buffer::Buffer2D, color::Color};

use super::Graphics;

impl Graphics {
    pub fn rectangle<T: Default + PartialEq + Copy + Clone + Debug>(
        target: &mut Buffer2D<T>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        fill: Option<T>,
        border: Option<T>,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        let y_start = y.min(target.height - 1);
        let y_end_inclusive = (y + height - 1).min(target.height - 1);

        let x_start = x.min(target.width - 1);
        let x_end_inclusive = (x + width - 1).min(target.width - 1);

        if y_start == y_end_inclusive || x_start == x_end_inclusive {
            return;
        }

        // Render a fill.

        if let Some(fill_color) = fill {
            for current_y in y_start..y_end_inclusive + 1 {
                target.horizontal_line_unsafe(x_start, x_end_inclusive, current_y, fill_color)
            }
        }

        // Render a border.

        if let Some(border_color) = border {
            if y_start == y {
                // Top edge was not clipped.
                target.horizontal_line_unsafe(x_start, x_end_inclusive, y_start, border_color);
            }

            if y_end_inclusive == y + height - 1 {
                // Bottom edge was not clipped.
                target.horizontal_line_unsafe(
                    x_start,
                    x_end_inclusive,
                    y_end_inclusive,
                    border_color,
                );
            }

            if x_start == x {
                // Left edge was not clipped.
                target.vertical_line_unsafe(x_start, y_start, y_end_inclusive, border_color);
            }

            if x_end_inclusive == x + width - 1 {
                // Right edge was not clipped.
                target.vertical_line_unsafe(
                    x_end_inclusive,
                    y_start,
                    y_end_inclusive,
                    border_color,
                );
            }
        }
    }

    pub fn clip_rectangle<T: Default + PartialEq + Copy + Clone + Debug>(
        target: &Buffer2D<T>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Option<(u32, u32, u32, u32)> {
        // Overlapping tests.

        if x >= target.width as i32 || (x + width as i32) < 0 {
            return None;
        }

        if y >= target.height as i32 || (y + height as i32) < 0 {
            return None;
        }

        let (left, top, right, bottom) = (
            x.max(0).min(target.width as i32 - 1),
            y.max(0).min(target.height as i32 - 1),
            (x + width as i32).max(0).min(target.width as i32 - 1),
            (y + height as i32).max(0).min(target.height as i32 - 1),
        );

        Some((
            left as u32,
            top as u32,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
    }

    pub fn rectangle_blended(
        target: &mut Buffer2D,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        fill: Option<Color>,
        border: Option<Color>,
    ) {
        if width == 0 || height == 0 {
            return;
        }

        let y_start = y.min(target.height - 1);
        let y_end_inclusive = (y + height - 1).min(target.height - 1);

        let x_start = x.min(target.width - 1);
        let x_end_inclusive = (x + width - 1).min(target.width - 1);

        if y_start == y_end_inclusive || x_start == x_end_inclusive {
            return;
        }

        // Render a fill.

        if let Some(fill_color) = fill {
            for current_y in y_start..y_end_inclusive + 1 {
                target.horizontal_line_blended_unsafe(
                    x_start,
                    x_end_inclusive,
                    current_y,
                    fill_color,
                )
            }
        }

        // Render a border.

        if let Some(border_color) = border {
            if y_start == y {
                // Top edge was not clipped.
                target.horizontal_line_blended_unsafe(
                    x_start,
                    x_end_inclusive,
                    y_start,
                    border_color,
                );
            }

            if y_end_inclusive == y + height - 1 {
                // Bottom edge was not clipped.
                target.horizontal_line_blended_unsafe(
                    x_start,
                    x_end_inclusive,
                    y_end_inclusive,
                    border_color,
                );
            }

            if x_start == x {
                // Left edge was not clipped.
                target.vertical_line_blended_unsafe(
                    x_start,
                    y_start,
                    y_end_inclusive,
                    border_color,
                );
            }

            if x_end_inclusive == x + width - 1 {
                // Right edge was not clipped.
                target.vertical_line_blended_unsafe(
                    x_end_inclusive,
                    y_start,
                    y_end_inclusive,
                    border_color,
                );
            }
        }
    }
}
