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

}
