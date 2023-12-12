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
		write!(v, "(r={}, g={}, b={}, a={})", self.r, self.g, self.b, self.a)
    }
}

impl Color
{

	pub const fn rgb(
		r: u8,
		g: u8,
		b: u8) -> Color
	{
		return Color { r, g, b, a: 0xff };
	}

	// pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
	// 	return Color { r, g, b, a }
	// }

	// @NOTE(mzalla) Check out:
	// https://doc.rust-lang.org/rust-by-example/conversion/from_into.html
	pub const fn to_u32(
		&self) -> u32
	{
		return
			(self.r as u32) |
			(self.g as u32) << 8 |
			(self.b as u32) << 16 |
			(self.a as u32) << 24;
	}

	pub fn to_vec3(
		&self) -> Vec3
	{
		return Vec3 {
			x: (self.r as f32) / 255.0,
			y: (self.g as f32) / 255.0,
			z: (self.b as f32) / 255.0,
		}
	}

}
