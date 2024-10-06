use crate::buffer::Buffer2D;

mod circle;
mod crosshair;
mod line;
mod rectangle;
pub mod text;

pub struct Graphics {}

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
