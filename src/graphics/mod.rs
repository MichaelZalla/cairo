use sdl2::pixels::Color as SDLColor;

use sdl2::ttf::Font;

use crate::{
	vec::vec2,
	color::Color,
};

#[derive(Clone)]
pub struct PixelBuffer {
	pub width: u32,
	pub height: u32,
	pub width_over_height: f32,
	pub height_over_width: f32,
	pub pixels: Vec<u32>,
}

impl PixelBuffer {

	pub fn clear(
		&mut self,
		color: Color) -> &Self
	{
		for i in 0..self.pixels.len() {
			self.pixels[i] = color.to_u32();
		}
		self
	}

}

#[derive(Clone)]
pub struct TextOperation<'a> {
	pub text: &'a String,
	pub x: u32,
	pub y: u32,
	pub color: Color,
}

#[derive(Clone)]
pub struct Graphics {
	pub buffer: PixelBuffer,
}

impl Graphics {

	pub fn get_pixel_data(&self) -> &Vec<u32> {
		return &self.buffer.pixels;
	}

	pub fn set_pixel(
		&mut self,
		x: u32,
		y: u32,
		color: Color)
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

	pub fn line(
		&mut self,
		mut x1: u32,
		mut y1: u32,
		mut x2: u32,
		mut y2: u32,
		color: Color)
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
		color: Color)
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

	pub fn text(
		&mut self,
		font: &Font,
		op: TextOperation) -> Result<(), String>
	{

		// Generate a new text rendering (surface)

		let surface = font
			.render(op.text)
			.blended(
				SDLColor::RGBA(
					op.color.r,
					op.color.g,
					op.color.b,
					op.color.a
				)
			)
			.map_err(|e| e.to_string())?;
		
		// Read the pixel data from the rendered surface 

		let text_surface_canvas = surface.into_canvas()?;

		let text_surface_canvas_size = text_surface_canvas.output_size()?;

		let text_canvas_width = text_surface_canvas_size.0;
		let text_canvas_height = text_surface_canvas_size.1;

		let text_surface_pixels = text_surface_canvas
			.read_pixels(None, sdl2::pixels::PixelFormatEnum::RGBA32)?;

		// Copy the rendered pixels to the graphics buffer, with padding

		for y in 0..text_canvas_height {
			for x in 0..text_canvas_width {

				let text_surface_pixels_index =
					(x as usize + y as usize * text_canvas_width as usize) * 4;

				let a = text_surface_pixels[text_surface_pixels_index + 3];

				if a != 0 {

					self.set_pixel(
						op.x + x,
						op.y + y,
						Color {
							r: text_surface_pixels[text_surface_pixels_index],
							g: text_surface_pixels[text_surface_pixels_index + 1],
							b: text_surface_pixels[text_surface_pixels_index + 2],
							a,
						},
					)

				}

			}
		}

		Ok(())

	}	

}
