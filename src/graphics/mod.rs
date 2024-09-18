use std::{
    cmp::{max, min},
    mem,
};

use crate::{buffer::Buffer2D, color::Color, vec::vec2};

pub mod text;

#[derive(Clone)]
pub struct Graphics {}

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

            vertical_line_unsafe(buffer, x1 as u32, min_y as u32, max_y as u32, color_u32);
        } else if y2 == y1 {
            // Horizontal line

            // dbg!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

            let min_x = min(x1, x2);
            let max_x = max(x1, x2);

            horizontal_line_unsafe(buffer, min_x as u32, max_x as u32, y1 as u32, color_u32);
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

    pub fn poly_line(buffer: &mut Buffer2D, p: &[vec2::Vec2], color: &Color) {
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

        Graphics::line(
            buffer,
            p[last_index].x as i32,
            p[last_index].y as i32,
            p[0].x as i32,
            p[0].y as i32,
            color,
        );
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
                horizontal_line_unsafe(
                    buffer,
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
                horizontal_line_unsafe(buffer, x_start, x_end_inclusive, y_start, color_u32);
            }

            if y_end_inclusive == y + height - 1 {
                // Bottom edge was not clipped.
                horizontal_line_unsafe(
                    buffer,
                    x_start,
                    x_end_inclusive,
                    y_end_inclusive,
                    color_u32,
                );
            }

            if x_start == x {
                // Left edge was not clipped.
                vertical_line_unsafe(buffer, x_start, y_start, y_end_inclusive, color_u32);
            }

            if x_end_inclusive == x + width - 1 {
                // Right edge was not clipped.
                vertical_line_unsafe(buffer, x_end_inclusive, y_start, y_end_inclusive, color_u32);
            }
        }
    }

    pub fn circle(
        buffer: &mut Buffer2D,
        center_x: u32,
        center_y: u32,
        radius: u32,
        fill: Option<&Color>,
        border: Option<&Color>,
    ) {
        assert!(
            fill.is_some() || border.is_some(),
            "Called `Graphics::circle()` with no fill or border provided!"
        );

        let fill_u32 = if let Some(fill) = fill {
            fill.to_u32()
        } else {
            0
        };

        let buffer_width_minus_one = buffer.width - 1;
        let buffer_height_minus_one = buffer.height - 1;

        // If no border was specified, use the fill color for perimeter.

        let border_u32 = border.unwrap_or_else(|| fill.as_ref().unwrap()).to_u32();

        // Begin at (+radius, 0), relative to the circle's center.

        let (mut x, mut y) = (radius as i32, 0 as i32);

        static DO_DDA_ALGORITHM: bool = true;

        let r_2 = (radius + radius) as i32;

        // d_y = -4y - 2;
        // d_y = -4(0) - 2;
        // d_y = 0 - 2;
        // d_y = -2;
        let mut d_y = -2;

        // d_x = 4x - 4;
        // d_x = 4(r) - 4;
        let mut d_x = r_2 + r_2 - 4;

        let mut decision_value = r_2 - 1;

        // {
        //     // Credit: Molly Rocket, "Efficient DDA Circle Outlines" (2023).
        //     //
        //     // See: https://youtu.be/JtgQJT08J1g

        //     let x_squared = up_x * up_x;

        //     let c = 2 * r_squared - 1;

        //     // |A| <? |B|
        //     // -A <? B
        //     // -((x-1)*(x-1) + y^2 - r^2)                    <? x*x + y^2 - r^2
        //     // -(x^2 - 2x + 1 + y^2 - r^2)                   <? x^2 + y^2 - r^2
        //     // -x^2 + 2x - 1 - y^2 + r^2                     <? x^2 + y^2 - r^2
        //     // -x^2 + 2x - 1 - y^2 + r^2 - (x^2 + y^2 - r^2) <? 0
        //     // -x^2 + 2x - 1 - y^2 + r^2 - x^2 - y^2 + r^2   <? 0
        //     // -2x^2 + 2x - 1 - 2y^2 + 2r^2                  <? 0
        //     // -2x^2 + 2x - 2y^2 + C                           <? 0

        //     decision_value = -2 * x_squared + 2 * left_x - 2 * y_squared + c;
        // }

        loop {
            let local_coordinates = [
                (x, y),
                (x, -y),
                (-x, y),
                (-x, -y),
                (y, x),
                (-y, x),
                (y, -x),
                (-y, -x),
            ];

            for (local_x, local_y) in local_coordinates {
                let global_x = (center_x as i32) + local_x;
                let global_y = (center_y as i32) + local_y;

                if global_y >= 0 && global_y < buffer.height as i32 {
                    // Border.

                    if global_x >= 0 && global_x < buffer.width as i32 {
                        buffer.set(global_x as u32, global_y as u32, border_u32);
                    }

                    // Fill.

                    let (mut x1, mut x2) = (center_x as i32 - local_x, global_x);

                    if x1 == x2 {
                        continue;
                    }

                    if x2 < x1 {
                        mem::swap(&mut x1, &mut x2);
                    }

                    x1 = x1.clamp(0, buffer_width_minus_one as i32);
                    x2 = x2.clamp(0, buffer_width_minus_one as i32);

                    let x2_minus_one = x2 - 1;

                    if local_x == x && (local_y == y || local_y == -y) {
                        horizontal_line_unsafe(
                            buffer,
                            x1 as u32,
                            x2_minus_one as u32,
                            global_y as u32,
                            fill_u32,
                        );
                    } else if local_x == y && (local_y == x || local_y == -x) {
                        if local_y == x {
                            if global_y > 0 {
                                horizontal_line_unsafe(
                                    buffer,
                                    x1 as u32,
                                    x2_minus_one as u32,
                                    (global_y - 1) as u32,
                                    fill_u32,
                                );
                            }
                        } else {
                            if global_y < buffer_height_minus_one as i32 {
                                horizontal_line_unsafe(
                                    buffer,
                                    x1 as u32,
                                    x2_minus_one as u32,
                                    (global_y + 1) as u32,
                                    fill_u32,
                                );
                            }
                        }
                    }
                }
            }

            let up_y = y + 1;
            let left_x = x - 1;

            let y_squared = up_y * up_y;

            if DO_DDA_ALGORITHM {
                decision_value += d_y;

                d_y -= 4;

                let should_go_left = decision_value < 0;

                y += 1;

                if should_go_left {
                    decision_value += d_x;

                    d_x -= 4;

                    x -= 1;
                }
            } else {
                let r_squared = (radius * radius) as i32;

                let up_x = x;

                let y_squared_minus_r_squared = y_squared - r_squared;

                // Explicit equation: x^2 + y^2 = r^2

                let left_error = left_x * left_x + y_squared_minus_r_squared;
                let up_error = up_x * up_x + y_squared_minus_r_squared;

                let should_go_left = -left_error < up_error;

                y += 1;

                if should_go_left {
                    x -= 1;
                }
            }

            if y >= x {
                break;
            }
        }
    }

    pub fn crosshair(
        buffer: &mut Buffer2D,
        x: i32,
        y: i32,
        length: u16,
        thickness: u16,
        mut gap: u16,
        center_dot: bool,
        color: &Color,
    ) {
        gap = gap.min((length as f32 / 2.0).ceil() as u16);

        for i in 0..thickness {
            let offset_x = x - (thickness as f32 / 2.0).ceil() as i32 + i as i32;
            let offset_y = y - (thickness as f32 / 2.0).ceil() as i32 + i as i32;

            // Horizontal segments
            Graphics::line(
                buffer,
                x - (length as f32 / 2.0).ceil() as i32,
                offset_y,
                x - gap as i32,
                offset_y,
                color,
            );

            Graphics::line(
                buffer,
                x + gap as i32,
                offset_y,
                x + (length as f32 / 2.0).ceil() as i32,
                offset_y,
                color,
            );

            // Vertical segments

            Graphics::line(
                buffer,
                offset_x,
                y - (length as f32 / 2.0).ceil() as i32,
                offset_x,
                y - gap as i32,
                color,
            );

            Graphics::line(
                buffer,
                offset_x,
                y + gap as i32,
                offset_x,
                y + (length as f32 / 2.0).ceil() as i32,
                color,
            );

            // Center dot

            if center_dot {
                Graphics::line(
                    buffer,
                    x - (thickness as f32 / 2.0).ceil() as i32,
                    offset_y,
                    x + (thickness as f32 / 2.0).ceil() as i32,
                    offset_y,
                    color,
                );
            }
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

// @NOTE Assumes all coordinate arguments lie inside the buffer boundary.
pub fn horizontal_line_unsafe(buffer: &mut Buffer2D, x1: u32, x2: u32, y: u32, color_u32: u32) {
    for x in x1..x2 + 1 {
        buffer.set(x, y, color_u32);
    }
}

// @NOTE Assumes all coordinate arguments lie inside the buffer boundary.
pub fn vertical_line_unsafe(buffer: &mut Buffer2D, x: u32, y1: u32, y2: u32, color_u32: u32) {
    for y in y1..y2 + 1 {
        buffer.set(x, y, color_u32);
    }
}
