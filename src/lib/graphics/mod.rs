use super::{color, vec::vec2};

#[derive(Clone)]
pub struct PixelBuffer {
	pub width: u32,
	pub height: u32,
	pub width_over_height: f32,
	pub height_over_width: f32,
	pub pixels: Vec<u32>,
}

impl PixelBuffer {

	pub fn clear(&mut self) -> &Self {
		for i in 0..self.pixels.len() {
			self.pixels[i] = 0;
		}
		self
	}

}

#[derive(Clone)]
pub struct Graphics {
	pub buffer: PixelBuffer,
}

impl Graphics {

	pub fn get_pixel_data(&self) -> &Vec<u32> {
		return &self.buffer.pixels;
	}

	#[inline(always)]
	pub fn set_pixel(
		&mut self,
		x: u32,
		y: u32,
		color: color::Color) -> ()
	{

		if x > (self.buffer.width - 1) || y > (self.buffer.pixels.len() as u32 / self.buffer.width as u32 - 1) {
			// panic!("Call to draw::set_pixel with invalid coordinate ({},{})!", x, y);
			return;
		}

		let pixel_index = (y * self.buffer.width + x) as usize;

		let r = color.r as u32;
		let g = (color.g as u32).rotate_left(8);
		let b = (color.b as u32).rotate_left(16);
		let a = (color.a as u32).rotate_left(24);

		self.buffer.pixels[pixel_index] = r|g|b|a;

	}

	// #[inline]
	pub fn line(
		&mut self,
		mut x1: u32,
		mut y1: u32,
		mut x2: u32,
		mut y2: u32,
		color: color::Color) -> ()
	{

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
				self.set_pixel(x1, y, color);
			}

		}
		else if y2 == y1 {

			// Horizontal line

			// dbg!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

			for x in x1..x2 {
				self.set_pixel(x, y1, color);
			}

		}
		else {

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
					self.set_pixel(((y as f32 - b) / m) as u32, y, color);
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
					self.set_pixel(x, (m * x as f32 + b) as u32, color);
				}

			}

		}

	}

	pub fn poly_line(
		&mut self,
		p: &[vec2::Vec2],
		color: color::Color) -> ()
	{

		for i in 0..p.len() {

			if i == p.len() - 1 {
				self.line(p[i].x as u32, p[i].y as u32, p[0].x as u32, p[0].y as u32, color);
			}
			else {
				self.line(p[i].x as u32, p[i].y as u32, p[i+1].x as u32, p[i+1].y as u32, color);
			}

		}

	}

}
