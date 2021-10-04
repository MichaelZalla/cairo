use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub static BLACK: Color = Color::RGB(0, 0, 0);
pub static WHITE: Color = Color::RGB(255, 255, 255);

impl fmt::Display for Color {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(v, "(r={}, g={}, b={}, a={})", self.r, self.g, self.b, self.a)
    }
}

impl Color
{
	pub const fn RGB(r: u8, g: u8, b: u8) -> Color {
		return Color { r, g, b, a: 0xff }
	}
	pub const fn RGBA(r: u8, g: u8, b: u8, a: u8) -> Color {
		return Color { r, g, b, a }
	}
}
