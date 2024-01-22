use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct PixelBuffer<T = u32>
where
    T: Default + PartialEq + Copy + Clone + Debug,
{
    pub width: u32,
    pub height: u32,
    pub width_over_height: f32,
    pub height_over_width: f32,
    pub data: Vec<T>,
}

impl<T> PixelBuffer<T>
where
    T: Default + PartialEq + Copy + Clone + Debug,
{
    pub fn new(width: u32, height: u32) -> Self {
        return PixelBuffer {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            height_over_width: height as f32 / width as f32,
            data: vec![Default::default(); (width * height) as usize],
        };
    }

    pub fn from_data(width: u32, height: u32, data: Vec<T>) -> Self {
        return PixelBuffer {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            height_over_width: height as f32 / width as f32,
            data,
        };
    }

    pub fn get_all(&self) -> &Vec<T> {
        return &self.data;
    }

    pub fn get(&self, x: u32, y: u32) -> &T {
        &self.data[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        if x > (self.width - 1) || y > (self.height - 1) {
            // panic!("Call to PixelBuffer.set() with invalid coordinate ({},{})!", x, y);
            return;
        }

        self.data[(y * self.width + x) as usize] = value;
    }

    pub fn set_raw(&mut self, index: usize, value: T, key_color: T) {
        debug_assert!(index < self.data.len());

        if value != key_color {
            self.data[index] = value;
        }
    }

    pub fn clear(&mut self, value: T) -> &Self {
        for i in 0..self.data.len() {
            self.data[i] = value;
        }
        self
    }

    pub fn blit(&mut self, left: u32, top: u32, width: u32, height: u32, pixels: &Vec<T>) -> () {
        debug_assert!(pixels.len() as u32 == width * height);

        for x in left..(left + width) {
            for y in top..(top + height) {
                let src_pixel_index = ((y - top) * width + (x - left)) as usize;

                let src_pixel_value = pixels[src_pixel_index];

                let dest_pixel_index = (y * self.width + x) as usize;

                self.data[dest_pixel_index] = src_pixel_value;
            }
        }
    }
}
