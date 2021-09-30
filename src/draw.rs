extern crate sdl2;

use sdl2::pixels;
use sdl2::render::Canvas;
use sdl2::video::Window;

use sdl2::gfx::primitives::DrawRenderer;

// #[inline]
pub fn line(
	canvas: &Canvas<Window>,
	mut x1: i16,
	mut y1: i16,
	mut x2: i16,
	mut y2: i16,
	color: pixels::Color
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

		println!("Drawing vertical line from ({},{}) to ({},{})!", x1, y1, x2, y2);

		for y in y1..y2 {
			let _ = canvas.pixel(x1, y, color);
		}

	}
	else if y2 == y1 {

		// Horizontal line

		println!("Drawing horizontal line from ({},{}) to ({},{})!", x1, y1, x2, y2);

		for x in x1..x2 {
			let _ = canvas.pixel(x, y1, color);
		}

	}
	else {

		let dx = x2 - x1;
		let dy = y2 - y1;
		let m = dy as f32 / dx as f32;
		let b = (y1 as f32 + y2 as f32 - m * (x1 + x2) as f32) / 2.0;

		// println!("m = {}, b = {}", m, b);

		if m.abs() > 1.0 {

			if y2 < y1 {
				let t: i16 = y1;
				y1 = y2;
				y2 = t;
			}

			// Vertical-ish line
			for y in y1..y2 {
				let _ = canvas.pixel(((y as f32 - b) / m) as i16, y, color);
			}

		}
		else {

			if x2 < x1 {
				let t: i16 = x1;
				x1 = x2;
				x2 = t;
			}

			// Horizontal-ish line
			for x in x1..x2 {
				let _ = canvas.pixel(x, (m * x as f32 + b) as i16, color);
			}

		}

	}

}
