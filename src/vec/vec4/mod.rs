use std::cmp;
use std::ops;
use std::fmt;

use super::vec3::Vec3;

#[derive(Debug, Copy, Clone, Default)]
pub struct Vec4 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

impl fmt::Display for Vec4 {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(v, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

impl Vec4 {

	pub fn new(
		v: Vec3,
		w: f32) -> Self
	{
		return Vec4 {
			x: v.x,
			y: v.y,
			z: v.z,
			w
		};
	}

}

impl cmp::PartialEq for Vec4 {
	fn eq(&self, other: &Self) -> bool {
        self.x == other.x &&
        self.y == other.y &&
        self.z == other.z &&
        self.w == other.w
    }
}

impl ops::Add<Vec4> for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4{
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
			w: self.w + rhs.w,
		}
    }
}

impl ops::AddAssign<Vec4> for Vec4 {
    fn add_assign(&mut self, rhs: Vec4) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
		self.w += rhs.w;
	}
}

impl ops::Sub<Vec4> for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4{
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
			w: self.w - rhs.w,
		}
    }
}

impl ops::SubAssign<Vec4> for Vec4 {
    fn sub_assign(&mut self, rhs: Vec4) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
		self.w -= rhs.w;
	}
}

impl ops::Mul<Vec4> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4{
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
			w: self.w * rhs.w,
		}
    }
}

impl ops::Mul<f32> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f32) -> Vec4 {
        Vec4{
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
			w: self.w * rhs,
		}
    }
}

impl ops::Div<f32> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: f32) -> Vec4 {
        Vec4{
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
			w: self.w / rhs,
		}
    }
}

impl ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
		self.w *= rhs;
	}
}

impl ops::MulAssign<Vec4> for Vec4 {
    fn mul_assign(&mut self, rhs: Vec4) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
		self.w *= rhs.w;
	}
}

impl Vec4 {

	pub fn mag(self) -> f32 {
		return ((self.x.powi(2) + self.y.powi(2) + self.z.powi(2)) / 2.0).sqrt();
	}

	pub fn dot(self, rhs: Vec4) -> f32 {
		// return self.mag() * rhs.mag() * theta.cos();
		return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
	}

	pub fn as_normal(self) -> Vec4 {
		let mag = self.mag();
		Vec4{
			x: self.x / mag,
			y: self.y / mag,
			z: self.z / mag,
			w: self.w,
		}
	}

	pub fn hadamard(
		&mut self,
		rhs: Vec4)
	{
		*self *= rhs;
	}

	pub fn get_hadamard(
		&self,
		v: Vec4) -> Vec4
	{
		let mut result = self.clone();

		result.hadamard(v);

		return result;
	}

	pub fn saturate(
		&mut self) -> &Self
	{
		self.x = self.x.max(0.0).min(1.0);
		self.y = self.y.max(0.0).min(1.0);
		self.z = self.z.max(0.0).min(1.0);

		return self;
	}

}