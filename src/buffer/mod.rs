use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Buffer2D<T = u32>
where
    T: Default + PartialEq + Copy + Clone + Debug,
{
    pub width: u32,
    pub height: u32,
    pub width_over_height: f32,
    pub height_over_width: f32,
    pub data: Vec<T>,
}

impl<T> Buffer2D<T>
where
    T: Default + PartialEq + Copy + Clone + Debug,
{
    pub fn new(width: u32, height: u32, fill_value: Option<T>) -> Self {
        let width_over_height = width as f32 / height as f32;
        let height_over_width = height as f32 / width as f32;

        let value: T = match fill_value {
            Some(fill_value) => fill_value,
            None => Default::default(),
        };

        let data: Vec<T> = vec![value; (width * height) as usize];

        Self {
            width,
            height,
            width_over_height,
            height_over_width,
            data,
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<T>) -> Self {
        Buffer2D {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            height_over_width: height as f32 / width as f32,
            data,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;

        self.height = height;

        self.width_over_height = width as f32 / height as f32;

        self.data
            .resize((width * height) as usize, Default::default());
    }

    pub fn get_all(&self) -> &Vec<T> {
        &self.data
    }

    pub fn get(&self, x: u32, y: u32) -> &T {
        debug_assert!(x < self.width && y < self.height);

        &self.data[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        debug_assert!(x < self.width && y < self.height);

        if x > (self.width - 1) || y > (self.height - 1) {
            // panic!("Call to Buffer2D.set() with invalid index coordinate ({},{})!", x, y);

            return;
        }

        self.data[(y * self.width + x) as usize] = value;
    }

    pub fn set_raw(&mut self, index: usize, value: T) {
        debug_assert!(index < self.data.len());

        self.data[index] = value;
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    pub fn clear(&mut self, value: Option<T>) -> &Self {
        let fill_value: T = match value {
            Some(value) => value,
            None => Default::default(),
        };

        for i in 0..self.data.len() {
            self.data[i] = fill_value;
        }

        self
    }

    pub fn blit(&mut self, left: u32, top: u32, width: u32, height: u32, data: &Vec<T>) -> () {
        debug_assert!(data.len() as u32 == width * height);

        for x in left..(left + width) {
            for y in top..(top + height) {
                let src_pixel_index = ((y - top) * width + (x - left)) as usize;

                let src_pixel_value = data[src_pixel_index];

                let dest_pixel_index = (y * self.width + x) as usize;

                self.data[dest_pixel_index] = src_pixel_value;
            }
        }
    }
}
