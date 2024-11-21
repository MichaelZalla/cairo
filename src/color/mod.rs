use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    animation::lerp,
    vec::{vec3::Vec3, vec4::Vec4},
};

pub mod blend;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub static TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);

pub static BLACK: Color = Color::rgb(0, 0, 0);
pub static DARK_GRAY: Color = Color::rgb(64, 64, 64);
pub static LIGHT_GRAY: Color = Color::rgb(124, 124, 124);
pub static WHITE: Color = Color::rgb(255, 255, 255);
pub static RED: Color = Color::rgb(255, 0, 0);
pub static YELLOW: Color = Color::rgb(255, 255, 0);
pub static ORANGE: Color = Color::rgb(255, 128, 0);
pub static GREEN: Color = Color::rgb(0, 255, 0);
pub static BLUE: Color = Color::rgb(0, 0, 255);
pub static SKY_BOX: Color = Color::rgb(102, 153, 255);

impl fmt::Display for Color {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            v,
            "(r={}, g={}, b={}, a={})",
            self.r, self.g, self.b, self.a
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseColorError;

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let channels: Vec<String> = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .map(|s| s.splitn(4, ','))
            .map(|s| s.map(|c| c.to_string()))
            .map(|s| s.collect())
            .ok_or(ParseColorError)?;

        debug_assert!(channels.len() >= 3);

        let r = channels[0].parse::<f32>().map_err(|_| ParseColorError)?;
        let g = channels[1].parse::<f32>().map_err(|_| ParseColorError)?;
        let b = channels[2].parse::<f32>().map_err(|_| ParseColorError)?;

        let a = if channels.len() > 3 {
            channels[3].parse::<f32>().map_err(|_| ParseColorError)?
        } else {
            255.0
        };

        Ok(Color { r, g, b, a })
    }
}

impl std::ops::MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.r = (self.r * rhs.r).clamp(0.0, 255.0);
        self.g = (self.g * rhs.g).clamp(0.0, 255.0);
        self.b = (self.b * rhs.b).clamp(0.0, 255.0);
        self.a = (self.a * rhs.a).clamp(0.0, 255.0);
    }
}

impl std::ops::Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        let mut cloned = self;

        cloned *= rhs;

        cloned
    }
}

impl std::ops::MulAssign<f32> for Color {
    fn mul_assign(&mut self, scale: f32) {
        self.r = (self.r * scale).clamp(0.0, 255.0);
        self.g = (self.g * scale).clamp(0.0, 255.0);
        self.b = (self.b * scale).clamp(0.0, 255.0);
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        let mut cloned = self;

        cloned *= scale;

        cloned
    }
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: 255.0,
        }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }

    pub fn to_u8(&self) -> (u8, u8, u8) {
        (self.r as u8, self.g as u8, self.b as u8)
    }

    pub const fn from_u32(bytes: u32) -> Self {
        Self {
            r: ((bytes) as u8) as f32,
            g: ((bytes >> 8) as u8) as f32,
            b: ((bytes >> 16) as u8) as f32,
            a: ((bytes >> 24) as u8) as f32,
        }
    }

    pub const fn to_u32(&self) -> u32 {
        (self.r as u32) | (self.g as u32) << 8 | (self.b as u32) << 16 | (self.a as u32) << 24
    }

    pub const fn from_vec3(color: Vec3) -> Self {
        Self {
            r: color.x,
            g: color.y,
            b: color.z,
            a: 255.0,
        }
    }

    pub const fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }

    pub const fn from_vec4(color: Vec4) -> Self {
        Self {
            r: color.x,
            g: color.y,
            b: color.z,
            a: color.w,
        }
    }

    pub const fn to_vec4(&self) -> Vec4 {
        Vec4 {
            x: self.r,
            y: self.g,
            z: self.b,
            w: self.a,
        }
    }

    pub fn lerp_linear(&self, rhs: Color, alpha: f32) -> Color {
        let start_vec3 = {
            let mut c = self.to_vec3();
            c.srgb_to_linear();
            c
        };

        let end_vec3: Vec3 = {
            let mut c = rhs.to_vec3();
            c.srgb_to_linear();
            c
        };

        let mut mixed = lerp(start_vec3, end_vec3, alpha);

        mixed.linear_to_srgb();

        Self::from_vec3(mixed)
    }
}

pub fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    // See: https://cs.stackexchange.com/a/127918

    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;

    let max = v;
    let c = s * v;
    let min = max - c;

    let h_prime = if h > 300.0 {
        (h - 360.0) / 60.0
    } else {
        h / 60.0
    };

    let (r, g, b) = match h_prime {
        -1.0..=1.0 => {
            if h_prime < 0.0 {
                let (r, g) = (max, min);
                let b = g - h_prime * c;
                (r, g, b)
            } else {
                let (r, b) = (max, min);
                let g = b + h_prime * c;
                (r, g, b)
            }
        }
        1.0..=3.0 => {
            if h_prime - 2.0 < 0.0 {
                let (g, b) = (max, min);
                let r = b - (h_prime - 2.0) * c;
                (r, g, b)
            } else {
                let (r, g) = (min, max);
                let b = r + (h_prime - 2.0) * c;
                (r, g, b)
            }
        }
        3.0..=5.0 => {
            if h_prime - 4.0 < 0.0 {
                let (r, b) = (min, max);
                let g = r - (h_prime - 4.0) * c;
                (r, g, b)
            } else {
                let (g, b) = (min, max);
                let r = g + (h_prime - 4.0) * c;
                (r, g, b)
            }
        }
        _ => (0.0, 0.0, 0.0),
    };

    Vec3 { x: r, y: g, z: b }
}
