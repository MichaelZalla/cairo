use crate::{buffer::Buffer2D, color::Color};

use super::Graphics;

impl Graphics {
    pub fn crosshair(
        buffer: &mut Buffer2D,
        x: i32,
        y: i32,
        length: u16,
        thickness: u16,
        mut gap: u16,
        center_dot: bool,
        color: Color,
    ) {
        gap = gap.min((length as f32 / 2.0).ceil() as u16);

        let color_u32 = color.to_u32();

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
                color_u32,
            );

            Graphics::line(
                buffer,
                x + gap as i32,
                offset_y,
                x + (length as f32 / 2.0).ceil() as i32,
                offset_y,
                color_u32,
            );

            // Vertical segments

            Graphics::line(
                buffer,
                offset_x,
                y - (length as f32 / 2.0).ceil() as i32,
                offset_x,
                y - gap as i32,
                color_u32,
            );

            Graphics::line(
                buffer,
                offset_x,
                y + gap as i32,
                offset_x,
                y + (length as f32 / 2.0).ceil() as i32,
                color_u32,
            );

            // Center dot

            if center_dot {
                Graphics::line(
                    buffer,
                    x - (thickness as f32 / 2.0).ceil() as i32,
                    offset_y,
                    x + (thickness as f32 / 2.0).ceil() as i32,
                    offset_y,
                    color_u32,
                );
            }
        }
    }
}
