extern crate sdl2;

pub struct PixelBuffer<'p> {
	pub pixels: &'p mut [u32],
	pub width: u32,
	pub bytes_per_pixel: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color
{
	pub const fn RGB(r: u8, g: u8, b: u8) -> Color {
		return Color { r, g, b, a: 0xff }
	}
	pub const fn RGBA(r: u8, g: u8, b: u8, a: u8) -> Color {
		return Color { r, g, b, a }
	}
}

#[inline(always)]
pub fn set_pixel(
	buffer: &mut PixelBuffer,
	x: u32,
	y: u32,
	color: Color,
) -> () {

	let pixel_index = (y * buffer.width + x) as usize;

	let r = color.r as u32;
	let g = (color.g as u32).rotate_left(8);
	let b = (color.b as u32).rotate_left(16);
	let a = (color.a as u32).rotate_left(24);

	buffer.pixels[pixel_index] = r|g|b|a;

}

#[inline]
pub fn line(
	buffer: &mut PixelBuffer,
	mut x1: u32,
	mut y1: u32,
	mut x2: u32,
	mut y2: u32,
	color: Color
) -> () {

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

		for y in y1..y2 {
			set_pixel(buffer, x1, y, color);
		}

	}
	else if y2 == y1 {

		// Horizontal line

		// dbg!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

		for x in x1..x2 {
			set_pixel(buffer, x, y1, color);
		}

	}
	else {

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
				set_pixel(buffer, ((y as f32 - b) / m) as u32, y, color);
			}

		}
		else {

			if x2 < x1 {
				let t: u32 = x1;
				x1 = x2;
				x2 = t;
			}

			// Horizontal-ish line
			for x in x1..x2 {
				set_pixel(buffer, x, (m * x as f32 + b) as u32, color);
			}

		}

	}

}
