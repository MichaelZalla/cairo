use std::cmp::{max, min};

use crate::{buffer::Buffer2D, color::Color, vec::vec2};

use super::Graphics;

impl Graphics {
    pub fn line(
        buffer: &mut Buffer2D,
        mut x1: i32,
        mut y1: i32,
        mut x2: i32,
        mut y2: i32,
        color: &Color,
    ) {
        if x1 as u32 >= buffer.width
            || x2 as u32 >= buffer.width
            || y1 as u32 >= buffer.height
            || y2 as u32 >= buffer.height
        {
            match clip_line(buffer, x1, y1, x2, y2) {
                Some((_x1, _y1, _x2, _y2)) => {
                    x1 = _x1;
                    y1 = _y1;
                    x2 = _x2;
                    y2 = _y2;
                }
                None => return,
            }
        }

        let color_u32 = color.to_u32();

        // y = m*x + b
        // x = (y - b) / m
        // m = (y2-y1)/(x2-x1)
        //
        // 1. y1 = m * x1 + b
        //    y2 = m * x2 + b
        //
        // 2. y1 + y2 = m * x1 + m * x2 + 2 * b
        //
        // 3. y1 + y2 - m * x1 - m * x2 = 2 * b
        //    y1 + y2 - m * (x1 + x2) = 2 * b
        //
        // 4. b = (y1 + y2 - m * (x1 + x2)) / 2
        //

        if x2 == x1 {
            // Vertical line

            // dbg!("Drawing vertical line from ({},{}) to ({},{})!", x1, y1, x2, y2);

            let min_y = min(y1, y2);
            let max_y = max(y1, y2);

            buffer.vertical_line_unsafe(x1 as u32, min_y as u32, max_y as u32, color_u32);
        } else if y2 == y1 {
            // Horizontal line

            // dbg!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

            let min_x = min(x1, x2);
            let max_x = max(x1, x2);

            buffer.horizontal_line_unsafe(min_x as u32, max_x as u32, y1 as u32, color_u32);
        } else {
            // println!("({}, {}), ({}, {})", x1, y1, x2, y2);

            let dx = x2 - x1;
            let dy = y2 - y1;

            let m = dy as f32 / dx as f32;
            let b = (y1 as f32 + y2 as f32 - m * (x1 + x2) as f32) / 2.0;

            // dbg!("m = {}, b = {}", m, b);

            if m.abs() > 1.0 {
                if y2 < y1 {
                    std::mem::swap(&mut y1, &mut y2);
                }

                // Vertical-ish line
                for y in y1..y2 + 1 {
                    buffer.set(((y as f32 - b) / m) as u32, y as u32, color_u32);
                }
            } else {
                if x2 < x1 {
                    std::mem::swap(&mut x1, &mut x2);
                }

                // Horizontal-ish line
                for x in x1..x2 + 1 {
                    buffer.set(x as u32, (m * x as f32 + b) as u32, color_u32);
                }
            }
        }
    }

    pub fn poly_line(buffer: &mut Buffer2D, p: &[vec2::Vec2], closed: bool, color: &Color) {
        if p.is_empty() {
            return;
        }

        let last_index: usize = p.len() - 1;

        for i in 0..last_index {
            Graphics::line(
                buffer,
                p[i].x as i32,
                p[i].y as i32,
                p[i + 1].x as i32,
                p[i + 1].y as i32,
                color,
            );
        }

        if closed {
            Graphics::line(
                buffer,
                p[last_index].x as i32,
                p[last_index].y as i32,
                p[0].x as i32,
                p[0].y as i32,
                color,
            );
        }
    }
}

fn clip_line(
    buffer: &Buffer2D,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) -> Option<(i32, i32, i32, i32)> {
    if x1 > x2 {
        let temp = (x2, y2);
        (x2, y2) = (x1, y1);
        (x1, y1) = temp;
    }

    // m = (y2 - y1) / (x2 - x1)
    let slope: f32 = (y2 - y1) as f32 / (x2 - x1) as f32;

    // y = mx + b
    // b = y - mx
    let bias: f32 = y1 as f32 - (slope * x1 as f32);

    if slope == f32::INFINITY {
        // Vertical line, safe to simply crop coordinates.

        return Some((
            (x1.max(0)).min(buffer.width as i32 - 1),
            (y1.max(0)).min(buffer.height as i32 - 1),
            (x2.max(0)).min(buffer.width as i32 - 1),
            (y2.max(0)).min(buffer.height as i32 - 1),
        ));
    }

    if x1 >= buffer.width as i32 {
        // y = mx + b
        x1 = (buffer.width - 1) as i32;
        y1 = (slope * x1 as f32 + bias) as i32;
    } else if x1 < 0 {
        // y = mx + b
        x1 = 0;
        y1 = (slope * x1 as f32 + bias) as i32;
    }

    if x2 >= buffer.width as i32 {
        // y = mx + b
        x2 = (buffer.width - 1) as i32;
        y2 = (slope * x2 as f32 + bias) as i32;
    } else if x2 < 0 {
        // y = mx + b
        x2 = 0_i32;
        y2 = (slope * x2 as f32 + bias) as i32;
    }

    if y1 >= buffer.height as i32 {
        // x = (y - b) / m
        y1 = (buffer.height - 1) as i32;
        x1 = ((y1 as f32 - bias) / slope) as i32;
    } else if y1 < 0 {
        // x = (y - b) / m
        y1 = 0_i32;
        x1 = ((y1 as f32 - bias) / slope) as i32;
    }

    if y2 >= buffer.height as i32 {
        // x = (y - b) / m
        y2 = (buffer.height - 1) as i32;
        x2 = ((y2 as f32 - bias) / slope) as i32;
    } else if y2 < 0 {
        // x = (y - b) / m
        y2 = 0_i32;
        x2 = ((y2 as f32 - bias) / slope) as i32;
    }

    if x1 >= 0
        && x1 < buffer.width as i32
        && x2 >= 0
        && x2 < buffer.width as i32
        && y1 >= 0
        && y1 < buffer.height as i32
        && y2 >= 0
        && y2 < buffer.height as i32
    {
        return Some((x1, y1, x2, y2));
    }

    None
}
