use std::ops::{Add, Mul, Sub};

pub fn lerp<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>>(
    start: T,
    end: T,
    alpha: f32,
) -> T {
    start + (end - start) * alpha
}

// See: https://www.desmos.com/calculator/8f1cpfqlmw
pub fn exponential<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>>(
    current: T,
    limit: T,
    rate: f32,
) -> T {
    current + (limit - current) * rate
}
