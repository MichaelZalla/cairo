use std::{
    cmp::{max, min},
    fmt::Debug,
    mem,
    ops::{Add, Div, Mul, Sub},
};

use crate::{buffer::Buffer2D, vec::vec2};

use super::Graphics;

impl Graphics {
    pub fn line<T>(
        target: &mut Buffer2D<T>,
        mut x1: i32,
        mut y1: i32,
        mut x2: i32,
        mut y2: i32,
        color: T,
    ) where
        T: Default
            + PartialEq
            + Copy
            + Clone
            + Debug
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    {
        if x1 as u32 >= target.width
            || x2 as u32 >= target.width
            || y1 as u32 >= target.height
            || y2 as u32 >= target.height
        {
            match clip_line(target.width, target.height, x1, y1, x2, y2) {
                Some(result) => {
                    x1 = result.left.0;
                    y1 = result.left.1;
                    x2 = result.right.0;
                    y2 = result.right.1;
                }
                None => return,
            }
        }

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

            target.vertical_line_unsafe(x1 as u32, min_y as u32, max_y as u32, color);
        } else if y2 == y1 {
            // Horizontal line

            // dbg!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

            let min_x = min(x1, x2);
            let max_x = max(x1, x2);

            target.horizontal_line_unsafe(min_x as u32, max_x as u32, y1 as u32, color);
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
                    target.set(((y as f32 - b) / m) as u32, y as u32, color);
                }
            } else {
                if x2 < x1 {
                    std::mem::swap(&mut x1, &mut x2);
                }

                // Horizontal-ish line
                for x in x1..x2 + 1 {
                    target.set(x as u32, (m * x as f32 + b) as u32, color);
                }
            }
        }
    }

    pub fn poly_line<T>(target: &mut Buffer2D<T>, p: &[vec2::Vec2], closed: bool, color: T)
    where
        T: Default
            + PartialEq
            + Copy
            + Clone
            + Debug
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>,
    {
        if p.is_empty() {
            return;
        }

        let last_index: usize = p.len() - 1;

        for i in 0..last_index {
            Graphics::line(
                target,
                p[i].x as i32,
                p[i].y as i32,
                p[i + 1].x as i32,
                p[i + 1].y as i32,
                color,
            );
        }

        if closed {
            Graphics::line(
                target,
                p[last_index].x as i32,
                p[last_index].y as i32,
                p[0].x as i32,
                p[0].y as i32,
                color,
            );
        }
    }
}

pub struct ClipLineResult {
    pub left: (i32, i32),
    pub right: (i32, i32),
    pub did_swap: bool,
}

pub fn clip_line(
    container_width: u32,
    container_height: u32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) -> Option<ClipLineResult> {
    let did_swap = if x1 > x2 {
        mem::swap(&mut x1, &mut x2);
        mem::swap(&mut y1, &mut y2);

        true
    } else {
        false
    };

    // m = (y2 - y1) / (x2 - x1)
    let slope: f32 = (y2 - y1) as f32 / (x2 - x1) as f32;

    // y = mx + b
    // b = y - mx
    let bias: f32 = y1 as f32 - (slope * x1 as f32);

    if slope == f32::INFINITY {
        // Vertical line, safe to simply crop coordinates.

        let (x1, y1) = (
            (x1.max(0)).min(container_width as i32 - 1),
            (y1.max(0)).min(container_height as i32 - 1),
        );

        let (x2, y2) = (
            (x2.max(0)).min(container_width as i32 - 1),
            (y2.max(0)).min(container_height as i32 - 1),
        );

        return Some(ClipLineResult {
            left: (x1, y1),
            right: (x2, y2),
            did_swap,
        });
    }

    if x1 >= container_width as i32 {
        // y = mx + b
        x1 = (container_width - 1) as i32;
        y1 = (slope * x1 as f32 + bias) as i32;
    } else if x1 < 0 {
        // y = mx + b
        x1 = 0;
        y1 = (slope * x1 as f32 + bias) as i32;
    }

    if x2 >= container_width as i32 {
        // y = mx + b
        x2 = (container_width - 1) as i32;
        y2 = (slope * x2 as f32 + bias) as i32;
    } else if x2 < 0 {
        // y = mx + b
        x2 = 0_i32;
        y2 = (slope * x2 as f32 + bias) as i32;
    }

    if y1 >= container_height as i32 {
        // x = (y - b) / m
        y1 = (container_height - 1) as i32;
        x1 = ((y1 as f32 - bias) / slope) as i32;
    } else if y1 < 0 {
        // x = (y - b) / m
        y1 = 0_i32;
        x1 = ((y1 as f32 - bias) / slope) as i32;
    }

    if y2 >= container_height as i32 {
        // x = (y - b) / m
        y2 = (container_height - 1) as i32;
        x2 = ((y2 as f32 - bias) / slope) as i32;
    } else if y2 < 0 {
        // x = (y - b) / m
        y2 = 0_i32;
        x2 = ((y2 as f32 - bias) / slope) as i32;
    }

    if x1 >= 0
        && x1 < container_width as i32
        && x2 >= 0
        && x2 < container_width as i32
        && y1 >= 0
        && y1 < container_height as i32
        && y2 >= 0
        && y2 < container_height as i32
    {
        Some(ClipLineResult {
            left: (x1, y1),
            right: (x2, y2),
            did_swap,
        })
    } else {
        None
    }
}
