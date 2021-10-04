use core::panic;

use super::vec::vec2;
use super::color;

pub struct PixelBuffer<'p> {
	pub pixels: &'p mut [u32],
	pub width: u32,
	// pub bytes_per_pixel: u32,
}

// #[inline(always)]
pub fn set_pixel(
	buffer: &mut PixelBuffer,
	x: u32,
	y: u32,
	color: color::Color) -> ()
{

	if x > (buffer.width - 1) || y > (buffer.pixels.len() as u32 / buffer.width as u32 - 1) {
		panic!("Call to draw::set_pixel with invalid coordinate ({},{})!", x, y);
	}

	let pixel_index = (y * buffer.width + x) as usize;

	let r = color.r as u32;
	let g = (color.g as u32).rotate_left(8);
	let b = (color.b as u32).rotate_left(16);
	let a = (color.a as u32).rotate_left(24);

	buffer.pixels[pixel_index] = r|g|b|a;

}

// #[inline]
pub fn line(
	buffer: &mut PixelBuffer,
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

pub fn poly_line(
	buffer: &mut PixelBuffer,
	p: &[vec2::Vec2],
	color: color::Color) -> ()
{

	for i in 0..p.len() {

		if i == p.len() - 1 {
			line(buffer, p[i].x as u32, p[i].y as u32, p[0].x as u32, p[0].y as u32, color);
		}
		else {
			line(buffer, p[i].x as u32, p[i].y as u32, p[i+1].x as u32, p[i+1].y as u32, color);
		}

	}

}

fn flat_top_triangle_fill(
	buffer: &mut PixelBuffer,
	p: &[vec2::Vec2],
	color: color::Color) -> ()
{

	let m0 = (p[2].x - p[0].x) / (p[2].y - p[0].y);
	let m1 = (p[2].x - p[1].x) / (p[2].y - p[1].y);

	let y_start = (p[0].y - 0.5).ceil() as u32;
	let y_end = (p[2].y - 0.5).ceil() as u32;

	// println!("Calling flat_top_triangle_fill: y_start={}, y_end={}", y_start, y_end);

	for y in y_start..y_end {

		let delta_y = (y as f32 + 0.5) - p[0].y;

		let x_left =  p[0].x + m0 * delta_y;
		let x_right = p[1].x + m1 * delta_y;

		let x_start = (x_left - 0.5).ceil() as u32;
		let x_end = (x_right - 0.5).ceil() as u32;

		for x in x_start..x_end {
			set_pixel(buffer, x, y, color);
		}

	}

}

fn flat_bottom_triangle_fill(
	buffer: &mut PixelBuffer,
	p: &[vec2::Vec2],
	color: color::Color) -> ()
{

	let m0 = (p[1].x - p[0].x) / (p[1].y - p[0].y);
	let m1 = (p[2].x - p[0].x) / (p[2].y - p[0].y);

	let y_start = (p[0].y - 0.5).ceil() as u32;
	let y_end = (p[2].y - 0.5).ceil() as u32;

	// println!("Calling flat_bottom_triangle_fill: y_start={}, y_end={}", y_start, y_end);

	for y in y_start..y_end {

		let delta_y = y as f32 + 0.5 - p[0].y;

		let x_left =  p[0].x + m0 * delta_y;
		let x_right = p[0].x + m1 * delta_y;

		let x_start = (x_left - 0.5).ceil() as u32;
		let x_end = (x_right - 0.5).ceil() as u32;

		for x in x_start..x_end {
			set_pixel(buffer, x, y, color);
		}

	}

}

pub fn triangle_fill(
	buffer: &mut PixelBuffer,
	p: &[vec2::Vec2],
	color: color::Color) -> ()
{

	let mut tri = vec![
		p[0],
		p[1],
		p[2],
	];

	// Sorts points by y-value (highest-to-lowest)

	if tri[1].y < tri[0].y {
		tri.swap(0, 1);
	}
	if tri[2].y < tri[1].y {
		tri.swap(1, 2);
	}
	if tri[1].y < tri[0].y {
		tri.swap(0, 1);
	}

	if tri[0].y == tri[1].y {

		// Flat-top (horizontal line is tri[0]-to-tri[1]);

		// tri[2] must sit below tri[0] and tri[1]; tri[0] and tri[1] cannot
		// have the same x-value; therefore, sort tri[0] and tri[1] by x-value;

		if tri[1].x < tri[0].x {
			tri.swap(0, 1);
		}

		flat_top_triangle_fill(
			buffer,
			tri.as_slice(),
			color);

		return;

	}
	else if tri[1].y == tri[2].y {

		// Flat-bottom (horizontal line is tri[1]-to-tri[2]);

		// tri[0] must sit above tri[1] and tri[2]; tri[1] and tri[2] cannot
		// have the same x-value; therefore, sort tri[1] and tri[2] by x-value;

		if tri[2].x < tri[1].x {
			tri.swap(1, 2);
		}

		flat_bottom_triangle_fill(
			buffer,
			tri.as_slice(),
			color);

		return;

	}
	else
	{

		// panic!("y0={}, y1={}, y2={}", tri[0].y, tri[1].y, tri[2].y);

		// Find splitting vertex

		let split_ratio =
			(tri[1].y - tri[0].y) /
			(tri[2].y - tri[0].y);

		let split_point = tri[0] + (tri[2] - tri[0]) * split_ratio;

		if tri[1].x < split_point.x {

			// Major right

			// tri[0] must sit above tri[1] and split_point; tri[1] and
			// split_point cannot have the same x-value; therefore, sort tri[1]
			// and split_point by x-value;

			flat_bottom_triangle_fill(
				buffer,
				vec![
					tri[0],
					tri[1],
					split_point,
				].as_slice(),
				color);

			flat_top_triangle_fill(
				buffer,
				vec![
					tri[1],
					split_point,
					tri[2],
				].as_slice(),
				color);

			return;

		}
		else
		{

			// Major left

			flat_bottom_triangle_fill(
				buffer,
				vec![
					tri[0],
					split_point,
					tri[1],
				].as_slice(),
				color);

			flat_top_triangle_fill(
				buffer,
				vec![
					split_point,
					tri[1],
					tri[2],
				].as_slice(),
				color);

			return;

		}

	}

}
