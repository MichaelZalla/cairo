use crate::color::Color;

#[derive(Clone)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub width_over_height: f32,
    pub height_over_width: f32,
    pub pixels: Vec<u32>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        return PixelBuffer {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            height_over_width: height as f32 / width as f32,
            pixels: vec![0 as u32; (width * height) as usize],
        };
    }

    pub fn get_pixel_data(&self) -> &Vec<u32> {
        return &self.pixels;
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x > (self.width - 1) || y > (self.pixels.len() as u32 / self.width as u32 - 1) {
            // panic!("Call to PixelBuffer.set_pixel() with invalid coordinate ({},{})!", x, y);
            return;
        }

        let pixel_index = (y * self.width + x) as usize;

        let r = color.r as u32;
        let g = (color.g as u32).rotate_left(8);
        let b = (color.b as u32).rotate_left(16);
        let a = (color.a as u32).rotate_left(24);

        self.pixels[pixel_index] = r | g | b | a;
    }

    pub fn clear(&mut self, color: Color) -> &Self {
        for i in 0..self.pixels.len() {
            self.pixels[i] = color.to_u32();
        }
        self
    }

    pub fn blit(&mut self, left: u32, top: u32, width: u32, height: u32, pixels: &Vec<u32>) -> () {
        debug_assert!(pixels.len() as u32 == width * height);

        for x in left..(left + width) {
            for y in top..(top + height) {
                let src_pixel_index = ((y - top) * width + (x - left)) as usize;

                let src_pixel_value = pixels[src_pixel_index];

                let dest_pixel_index = (y * self.width + x) as usize;

                self.pixels[dest_pixel_index] = src_pixel_value;
            }
        }
    }
}
