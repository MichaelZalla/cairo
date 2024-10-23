use std::{
    fmt::Debug,
    mem,
    ops::{Add, Div, Mul, Sub},
};

use crate::buffer::Buffer2D;

use super::Graphics;

impl Graphics {
    pub fn circle<T>(
        buffer: &mut Buffer2D<T>,
        center_x: u32,
        center_y: u32,
        radius: u32,
        fill: Option<T>,
        border: Option<T>,
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
        assert!(
            fill.is_some() || border.is_some(),
            "Called `Graphics::circle()` with no fill or border provided!"
        );

        let buffer_width_minus_one = buffer.width - 1;
        let buffer_height_minus_one = buffer.height - 1;

        // If no border was specified, use the fill color for perimeter.

        let border_value = border.unwrap_or_else(|| fill.unwrap());

        // Begin at (+radius, 0), relative to the circle's center.

        let (mut x, mut y) = (radius as i32, 0);

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

                    if global_x >= 0 && global_x < buffer.width as i32 && border.is_some() {
                        buffer.set(global_x as u32, global_y as u32, border_value);
                    }

                    // Fill.

                    if let Some(fill_color) = fill {
                        let (mut x1, mut x2) = (center_x as i32 - local_x, global_x);

                        if x1 == x2 {
                            continue;
                        }

                        if x2 < x1 {
                            mem::swap(&mut x1, &mut x2);
                        }

                        x1 = x1.clamp(0, buffer_width_minus_one as i32);
                        x2 = x2.clamp(0, buffer_width_minus_one as i32);

                        if local_x == x && (local_y == y || local_y == -y) {
                            buffer.horizontal_line_unsafe(
                                x1 as u32,
                                x2 as u32 - 1,
                                global_y as u32,
                                fill_color,
                            );
                        } else if local_x == y && (local_y == x || local_y == -x) {
                            if local_y == x {
                                if global_y > 0 {
                                    buffer.horizontal_line_unsafe(
                                        x1 as u32,
                                        x2 as u32,
                                        (global_y - 1) as u32,
                                        fill_color,
                                    );
                                }
                            } else if global_y < buffer_height_minus_one as i32 {
                                buffer.horizontal_line_unsafe(
                                    x1 as u32,
                                    x2 as u32,
                                    (global_y + 1) as u32,
                                    fill_color,
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

            if y > x {
                break;
            }
        }
    }
}
