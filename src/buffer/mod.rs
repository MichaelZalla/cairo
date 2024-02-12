use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use crate::color::blend::{self, BlendMode};

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

    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut T {
        debug_assert!(x < self.width && y < self.height);

        &mut self.data[(y * self.width + x) as usize]
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

    pub fn blit(&mut self, left: u32, top: u32, width: u32, height: u32, data: &Vec<T>) {
        self.blit_blended(left, top, width, height, data, None, None)
    }

    pub fn blit_blended(
        &mut self,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        data: &Vec<T>,
        blend_mode: Option<BlendMode>,
        blend_mode_max_value: Option<T>,
    ) {
        debug_assert!(data.len() as u32 == width * height);

        let right = (left + width - 1).min(self.width - 1);
        let bottom = (top + height - 1).min(self.height - 1);

        for x in left..right + 1 {
            for y in top..bottom + 1 {
                let dest_pixel_index = (y * self.width + x) as usize;
                let src_pixel_index = ((y - top) * width + (x - left)) as usize;

                let lhs = self.data[dest_pixel_index];
                let rhs = data[src_pixel_index];

                let result = match &blend_mode {
                    Some(mode) => blend::blend::<T>(mode, blend_mode_max_value, &lhs, &rhs),
                    None => blend::blend::<T>(&BlendMode::Normal, None, &lhs, &rhs),
                };

                self.data[dest_pixel_index] = result;
            }
        }
    }

    pub fn blit_from(&mut self, left: u32, top: u32, other: &Buffer2D<T>) {
        self.blit_blended_from(left, top, other, None, None)
    }

    pub fn blit_blended_from(
        &mut self,
        left: u32,
        top: u32,
        other: &Buffer2D<T>,
        blend_mode: Option<BlendMode>,
        blend_mode_max_value: Option<T>,
    ) {
        self.blit_blended(
            left,
            top,
            other.width,
            other.height,
            other.get_all(),
            blend_mode,
            blend_mode_max_value,
        )
    }
}
