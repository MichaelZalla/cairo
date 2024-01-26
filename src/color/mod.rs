use std::fmt;

use super::vec::vec3::Vec3;

#[derive(Default, Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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
        return Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: 255.0,
        };
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        return Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        };
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
        return (self.r as u32)
            | (self.g as u32) << 8
            | (self.b as u32) << 16
            | (self.a as u32) << 24;
    }

    pub fn from_vec3(color: Vec3) -> Self {
        Self {
            r: color.x,
            g: color.y,
            b: color.z,
            a: 255.0,
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        return Vec3 {
            x: self.r,
            y: self.g,
            z: self.b,
        };
    }
}
