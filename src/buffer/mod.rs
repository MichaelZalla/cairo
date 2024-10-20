use std::{
    fmt::Debug,
    mem::size_of,
    ops::{Add, Div, Mul, Sub},
    ptr,
};

use crate::{
    animation::lerp,
    color::{
        blend::{self, blend, BlendMode},
        Color,
    },
    vec::vec3::Vec3,
};

pub mod framebuffer;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Buffer2D<T = u32>
where
    T: Default + PartialEq + Copy + Clone + Debug,
{
    pub width: u32,
    pub height: u32,
    pub width_over_height: f32,
    pub center: Vec3,
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
        let value: T = fill_value.unwrap_or_default();

        let data: Vec<T> = vec![value; (width * height) as usize];

        Self {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            center: Vec3::from_x_y(width as f32 / 2.0, height as f32 / 2.0),
            data,
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<T>) -> Self {
        Buffer2D {
            width,
            height,
            width_over_height: width as f32 / height as f32,
            center: Vec3::from_x_y(width as f32 / 2.0, height as f32 / 2.0),
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

    pub fn get_all_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub fn get(&self, x: u32, y: u32) -> &T {
        debug_assert!(x < self.width && y < self.height);

        &self.data[(y * self.width + x) as usize]
    }

    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut T {
        debug_assert!(x < self.width && y < self.height);

        &mut self.data[(y * self.width + x) as usize]
    }

    pub fn get_raw(&self, index: usize) -> &T {
        debug_assert!(index < self.data.len());

        &self.data[index]
    }

    pub fn get_raw_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.data.len());

        &mut self.data[index]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        self.set_blended(x, y, value, BlendMode::Normal, None)
    }

    pub fn set_blended(
        &mut self,
        x: u32,
        y: u32,
        value: T,
        blend_mode: BlendMode,
        blend_mode_max_value: Option<T>,
    ) {
        debug_assert!(x < self.width && y < self.height);

        if x > (self.width - 1) || y > (self.height - 1) {
            // panic!("Call to Buffer2D.set() with invalid index coordinate ({},{})!", x, y);

            return;
        }

        let index = (y * self.width + x) as usize;
        let lhs = self.data[index];
        let rhs = value;

        self.data[index] = blend(&blend_mode, blend_mode_max_value, &lhs, &rhs);
    }

    pub fn set_raw(&mut self, index: usize, value: T) {
        self.set_raw_blended(index, value, BlendMode::Normal, None)
    }

    pub fn set_raw_blended(
        &mut self,
        index: usize,
        value: T,
        blend_mode: BlendMode,
        blend_mode_max_value: Option<T>,
    ) {
        debug_assert!(index < self.data.len());

        let lhs = self.data[index];
        let rhs = value;

        self.data[index] = blend(&blend_mode, blend_mode_max_value, &lhs, &rhs);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    pub fn clear(&mut self, value: Option<T>) {
        let fill_value: T = value.unwrap_or_default();

        self.data.fill(fill_value);
    }

    pub fn blit(&mut self, left: u32, top: u32, width: u32, height: u32, data: &[T]) {
        self.blit_blended(left, top, width, height, data, None, None)
    }

    pub fn blit_blended(
        &mut self,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        data: &[T],
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

impl Buffer2D<u32> {
    pub fn as_cast_slice<T, C>(&self, mut callback: C)
    where
        C: FnMut(&[T]),
    {
        let pixels_u32 = self.get_all().as_slice();

        unsafe {
            let pixels_t_const = ptr::slice_from_raw_parts(
                pixels_u32.as_ptr() as *const T,
                pixels_u32.len() * (size_of::<u32>() / size_of::<T>()),
            );

            let pixels_t_slice = &(*pixels_t_const);

            callback(pixels_t_slice);
        }
    }

    pub fn copy_to<T: Copy>(&self, target: &mut [T]) {
        self.as_cast_slice(|data| {
            target.copy_from_slice(data);
        });
    }

    pub fn horizontal_line_unsafe(&mut self, x1: u32, x2: u32, y: u32, value: u32) {
        // Assumes all coordinate arguments lie inside the buffer boundary.

        for x in x1..x2 + 1 {
            self.set(x, y, value);
        }
    }

    pub fn horizontal_line_blended_unsafe(&mut self, x1: u32, x2: u32, y: u32, color: &Color) {
        // Assumes all coordinate arguments lie inside the buffer boundary.

        static ONE_OVER_255: f32 = 1.0 / 255.0;

        let rhs = color.to_vec3() * ONE_OVER_255;

        let alpha = color.a * ONE_OVER_255;

        for x in x1..x2 + 1 {
            let lhs = Color::from_u32(*self.get(x, y)).to_vec3() * ONE_OVER_255;

            let blended = lerp(lhs, rhs, alpha);

            self.set(x, y, Color::from_vec3(blended * 255.0).to_u32());
        }
    }

    pub fn vertical_line_unsafe(&mut self, x: u32, y1: u32, y2: u32, value: u32) {
        // Assumes all coordinate arguments lie inside the buffer boundary.

        for y in y1..y2 + 1 {
            self.set(x, y, value);
        }
    }

    pub fn vertical_line_blended_unsafe(&mut self, x: u32, y1: u32, y2: u32, color: &Color) {
        // Assumes all coordinate arguments lie inside the buffer boundary.

        static ONE_OVER_255: f32 = 1.0 / 255.0;

        let rhs = color.to_vec3() * ONE_OVER_255;

        for y in y1..y2 + 1 {
            let lhs = Color::from_u32(*self.get(x, y)).to_vec3() * ONE_OVER_255;

            let blended = lerp(lhs, rhs, color.a);

            self.set(x, y, Color::from_vec3(blended * 255.0).to_u32());
        }
    }
}
