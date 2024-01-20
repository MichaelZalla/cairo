use std::cmp::{max, min};

use crate::{color::Color, vec::vec2};

use self::pixelbuffer::PixelBuffer;

pub mod pixelbuffer;
pub mod text;
#[derive(Clone)]
pub struct Graphics {
    pub buffer: PixelBuffer,
}

impl Graphics {
    pub fn line(&mut self, mut x1: u32, mut y1: u32, mut x2: u32, mut y2: u32, color: Color) {
        // y = m*x + b
        // x = (y - b) / m
        // m = (y2-y1)/(x2-x1)
        //
        // 1. y1 = m*x1 + b
        // y2 = m*x2 + b
        //
        // 2. y1 + y2 = m*x1 + m*x2 + 2*b
        //
        // 3. y1 + y2 - m*x1 - m*x2 = 2*b
        // y1 + y2 - m*(x1 + x2) = 2*b
        //
        // 4. b = (y1 + y2 - m*(x1 + x2)) / 2
        //

        if x2 == x1 {
            // Vertical line

            // dbg!("Drawing vertical line from ({},{}) to ({},{})!", x1, y1, x2, y2);

            let min_y = min(y1, y2);
            let max_y = max(y1, y2);

            for y in min_y..max_y {
                self.buffer.set_pixel(x1, y, color);
            }
        } else if y2 == y1 {
            // Horizontal line

            // dbg!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

            let min_x = min(x1, x2);
            let max_x = max(x1, x2);

            for x in min_x..max_x {
                self.buffer.set_pixel(x, y1, color);
            }
        } else {
            // println!("({}, {}), ({}, {})", x1, y1, x2, y2);

            let dx = x2 as i32 - x1 as i32;
            let dy = y2 as i32 - y1 as i32;
            let m = dy as f32 / dx as f32;
            let b = (y1 as f32 + y2 as f32 - m * (x1 + x2) as f32) / 2.0;

            // dbg!("m = {}, b = {}", m, b);

            if m.abs() > 1.0 {
                if y2 < y1 {
                    let t: u32 = y1;
                    y1 = y2;
                    y2 = t;
                }

                // Vertical-ish line
                for y in y1..y2 {
                    self.buffer.set_pixel(((y as f32 - b) / m) as u32, y, color);
                }
            } else {
                if x2 < x1 {
                    let t: u32 = x1;
                    x1 = x2;
                    x2 = t;
                }

                // Horizontal-ish line
                for x in x1..x2 {
                    self.buffer.set_pixel(x, (m * x as f32 + b) as u32, color);
                }
            }
        }
    }

    pub fn poly_line(&mut self, p: &[vec2::Vec2], color: Color) {
        for i in 0..p.len() {
            if i == p.len() - 1 {
                self.line(
                    p[i].x as u32,
                    p[i].y as u32,
                    p[0].x as u32,
                    p[0].y as u32,
                    color,
                );
            } else {
                self.line(
                    p[i].x as u32,
                    p[i].y as u32,
                    p[i + 1].x as u32,
                    p[i + 1].y as u32,
                    color,
                );
            }
        }
    }
}
