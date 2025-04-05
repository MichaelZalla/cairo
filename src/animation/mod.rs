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

pub fn smooth_step(start: f32, end: f32, value: f32) -> f32 {
    let alpha = (value - start) / (end - start);

    let clamped_alpha = alpha.clamp(0.0, 1.0);

    clamped_alpha * clamped_alpha * (3.0 - 2.0 * clamped_alpha)
}
