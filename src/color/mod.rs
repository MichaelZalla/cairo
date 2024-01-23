use std::fmt;

use super::vec::vec3::Vec3;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub static BLACK: Color = Color::rgb(0, 0, 0);
pub static WHITE: Color = Color::rgb(255, 255, 255);

pub static RED: Color = Color::rgb(255, 0, 0);
pub static GREEN: Color = Color::rgb(0, 255, 0);
pub static BLUE: Color = Color::rgb(0, 0, 255);

pub static YELLOW: Color = Color::rgb(255, 255, 0);

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

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        return Color { r, g, b, a: 0xff };
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        return Color { r, g, b, a };
    }

    pub const fn from_u32(bytes: u32) -> Self {
        Self {
            r: (bytes) as u8,
            g: (bytes >> 8) as u8,
            b: (bytes >> 16) as u8,
            a: (bytes >> 24) as u8,
        }
    }

    pub const fn to_u32(&self) -> u32 {
        return (self.r as u32)
            | (self.g as u32) << 8
            | (self.b as u32) << 16
            | (self.a as u32) << 24;
    }

    pub fn from_vec3(color: Vec3) -> Self {
        Self {
            r: (color.x * 255.0) as u8,
            g: (color.y * 255.0) as u8,
            b: (color.z * 255.0) as u8,
            a: 255 as u8,
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        return Vec3 {
            x: self.r as f32,
            y: self.g as f32,
            z: self.b as f32,
        };
    }
}
