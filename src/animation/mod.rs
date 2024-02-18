use std::ops::{Add, Mul, Sub};

pub fn lerp<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>>(
    start: T,
    end: T,
    alpha: f32,
) -> T {
    start + (end - start) * alpha
}
