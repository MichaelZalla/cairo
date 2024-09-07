use std::fmt;

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

impl std::ops::MulAssign<f32> for Color {
    fn mul_assign(&mut self, scale: f32) {
        self.r = (self.r * scale).max(0.0).min(255.0);
        self.g = (self.g * scale).max(0.0).min(255.0);
        self.b = (self.b * scale).max(0.0).min(255.0);
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        let mut cloned = self.clone();

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
