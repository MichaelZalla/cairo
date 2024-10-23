use std::{
    fmt::Debug,
    mem::size_of,
    ops::{Add, AddAssign, Div, Mul, Sub},
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
    T: Default + PartialEq + Copy + Clone + Debug,
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

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
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

    pub fn get_at(&self, index: usize) -> &T {
        debug_assert!(index < self.data.len());

        &self.data[index]
    }

    pub fn get_at_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.data.len());

        &mut self.data[index]
    }

    pub fn set_at(&mut self, index: usize, value: T) {
        assert!(index < self.data.len());

        self.data[index] = value;
    }

    pub fn set_at_unsafe(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        debug_assert!(x < self.width && y < self.height);

        if x > (self.width - 1) || y > (self.height - 1) {
            // panic!("Call to Buffer2D.set() with invalid index coordinate ({},{})!", x, y);

            return;
        }

        let index = (y * self.width + x) as usize;

        self.set_at_unsafe(index, value);
    }

    pub fn as_cast_slice<B, C>(&self, mut callback: C)
    where
        C: FnMut(&[B]),
    {
        let pixels_as_t = self.get_all().as_slice();

        unsafe {
            let pixels_as_b_const = ptr::slice_from_raw_parts(
                pixels_as_t.as_ptr() as *const B,
                pixels_as_t.len() * (size_of::<T>() / size_of::<B>()),
            );

            let pixels_as_b_slice = &(*pixels_as_b_const);

            callback(pixels_as_b_slice);
        }
    }

    pub fn copy(&mut self, source: &[T]) {
        for index in 0..self.data.len() {
            self.data[index] = source[index];
        }
    }

    pub fn copy_to<B: Copy>(&self, target: &mut [B]) {
        self.as_cast_slice(|data| {
            target.copy_from_slice(data);
        });
    }

    pub fn clear(&mut self, value: Option<T>) {
        let fill_value: T = value.unwrap_or_default();

        self.data.fill(fill_value);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;

        self.height = height;

        self.width_over_height = width as f32 / height as f32;

        self.data
            .resize((width * height) as usize, Default::default());
    }

    pub fn blit(&mut self, left: u32, top: u32, width: u32, height: u32, source: &[T]) {
        debug_assert!(source.len() as u32 == width * height);

        let right = (left + width - 1).min(self.width - 1);
        let bottom = (top + height - 1).min(self.height - 1);

        for x in left..right + 1 {
            for y in top..bottom + 1 {
                let dest_pixel_index = (y * self.width + x) as usize;
                let src_pixel_index = ((y - top) * width + (x - left)) as usize;

                self.data[dest_pixel_index] = source[src_pixel_index];
            }
        }
    }

    pub fn blit_from(&mut self, left: u32, top: u32, other: &Buffer2D<T>) {
        self.blit(left, top, other.width, other.height, other.get_all())
    }

    pub fn vertical_line_unsafe(&mut self, x: u32, y1: u32, y2: u32, value: T) {
        // Assumes all coordinate arguments lie inside the buffer boundary.

        for y in y1..y2 + 1 {
            self.set(x, y, value);
        }
    }

    pub fn horizontal_line_unsafe(&mut self, x1: u32, x2: u32, y: u32, value: T) {
        // Assumes all coordinate arguments lie inside the buffer boundary.

        for x in x1..x2 + 1 {
            self.set(x, y, value);
        }
    }
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
    pub fn set_blended(
        &mut self,
        x: u32,
        y: u32,
        value: T,
        blend_mode: BlendMode,
        blend_mode_max_value: Option<T>,
    ) {
        let index = (y * self.width + x) as usize;

        let lhs = &self.data[index];
        let rhs = &value;

        let blended = blend(&blend_mode, blend_mode_max_value, lhs, rhs);

        self.set_at_unsafe(index, blended);
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

    pub fn copy_blended(
        &mut self,
        source: &[T],
        blend_mode: Option<BlendMode>,
        blend_mode_max_value: Option<T>,
    ) {
        for (lhs, rhs) in std::iter::zip(&mut self.data, source) {
            let result = match &blend_mode {
                Some(mode) => blend::<T>(mode, blend_mode_max_value, lhs, rhs),
                None => blend::<T>(&BlendMode::Normal, None, lhs, rhs),
            };

            *lhs = result;
        }
    }

    pub fn blit_blended(
        &mut self,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
        source: &[T],
        blend_mode: Option<BlendMode>,
        blend_mode_max_value: Option<T>,
    ) {
        debug_assert!(source.len() as u32 == width * height);

        let right = (left + width - 1).min(self.width - 1);
        let bottom = (top + height - 1).min(self.height - 1);

        for x in left..right + 1 {
            for y in top..bottom + 1 {
                let dest_pixel_index = (y * self.width + x) as usize;
                let src_pixel_index = ((y - top) * width + (x - left)) as usize;

                let lhs = &self.data[dest_pixel_index];
                let rhs = &source[src_pixel_index];

                let result = match &blend_mode {
                    Some(mode) => blend::blend::<T>(mode, blend_mode_max_value, lhs, rhs),
                    None => blend::blend::<T>(&BlendMode::Normal, None, lhs, rhs),
                };

                self.data[dest_pixel_index] = result;
            }
        }
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

    pub fn dilate(&self, dest: &mut Buffer2D<T>, key_color: T, outline_color: T) {
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let color = self.get(x as u32, y as u32);

                if *color != key_color {
                    dest.set(x as u32, y as u32, *color);

                    for (index, (n_x, n_y)) in get_3x3_coordinates(x, y).iter().enumerate() {
                        if index == 4 {
                            // Skips center coordinate (4).
                            continue;
                        }

                        // Perform bounds-checking.
                        if *n_x < 0
                            || *n_x > (self.width - 1) as i32
                            || *n_y < 0
                            || *n_y > (self.height - 1) as i32
                        {
                            continue;
                        }

                        // Perform dilation (but only outside of the drawn objects).
                        if *self.get(*n_x as u32, *n_y as u32) == key_color {
                            dest.set(*n_x as u32, *n_y as u32, outline_color)
                        }
                    }
                }
            }
        }
    }
}

impl<T> Buffer2D<T>
where
    T: Default
        + PartialEq
        + Copy
        + Clone
        + Debug
        + Add<Output = T>
        + AddAssign
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + Mul<f32, Output = T>,
{
    pub fn blur(&self, dest: &mut Buffer2D<T>, weights: &[f32; 5], strength: u8, horizontal: bool) {
        let weights_0 = weights[0];

        for y in 0..self.height {
            for x in 0..self.width {
                let mut result = *self.get(x, y) * weights_0;

                for i in 1..strength {
                    let i_u32 = i as u32;

                    if horizontal {
                        if x >= i_u32 {
                            result += *self.get(x - i_u32, y) * weights[i as usize];
                        }
                        if x + i_u32 < self.width {
                            result += *self.get(x + i_u32, y) * weights[i as usize];
                        }
                    } else {
                        if y >= i_u32 {
                            result += *self.get(x, y - i_u32) * weights[i as usize];
                        }
                        if y + i_u32 < self.height {
                            result += *self.get(x, y + i_u32) * weights[i as usize];
                        }
                    }
                }

                dest.set(x, y, result);
            }
        }
    }
}

impl Buffer2D<u32> {
    pub fn horizontal_line_blended_unsafe(&mut self, x1: u32, x2: u32, y: u32, color: Color) {
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

    pub fn vertical_line_blended_unsafe(&mut self, x: u32, y1: u32, y2: u32, color: Color) {
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

pub fn get_3x3_coordinates(x: i32, y: i32) -> [(i32, i32); 9] {
    [
        // 0. Top-left
        (x - 1, y - 1),
        // 1. Above
        (x, y - 1),
        // 2. Top-right
        (x + 1, y - 1),
        // 3. Left
        (x - 1, y),
        // 4. Center
        (x, y),
        // 5. Right
        (x + 1, y),
        // 6. Bottom-left
        (x - 1, y + 1),
        // 7. Below
        (x, y + 1),
        // 8. Bottom-right
        (x + 1, y + 1),
    ]
}
