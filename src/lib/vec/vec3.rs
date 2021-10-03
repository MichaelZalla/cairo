use std::ops;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl fmt::Display for Vec3 {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(v, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3{
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3{
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
		}
    }
}

impl ops::MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
	}
}

impl Vec3 {

	fn len(self) -> f32 {
		return ((self.x.powi(2) + self.y.powi(2) + self.z.powi(2)) / 2.0).sqrt();
	}

	fn dot(self, rhs: Vec3) -> f32 {
		return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
	}

	fn as_normal(self) -> Vec3 {
		let len = self.len();
		Vec3{
			x: self.x / len,
			y: self.y / len,
			z: self.z / len,
		}
	}

	fn normalize(&mut self) -> () {
		let len = self.len();
		self.x /= len;
		self.y /= len;
		self.z /= len;
	}

	pub fn rotate_along_z(&mut self, phi: f32) -> () {
		let (x, y) = (self.x, self.y);
		self.x = (x * phi.cos()) - (y * phi.sin());
		self.y = (x * phi.sin()) + (y * phi.cos());
	}

	pub fn rotate_along_x(&mut self, phi: f32) -> () {
		// X-axis rotation looks like Z-axis rotation if replace:
		// - X axis with Y axis
		// - Y axis with Z axis
		// - Z axis with X axis
		let (y, z) = (self.y, self.z);
		self.y = y * phi.cos() - z * phi.sin();
		self.z = y * phi.sin() + z * phi.cos();
	}

	pub fn rotate_along_y(&mut self, phi: f32) -> () {
		// Y-axis rotation looks like Z-axis rotation if replace:
		// - X axis with Z axis
		// - Y axis with X axis
		// - Zaxis with Y axis
		let (z, x) = (self.z, self.x);
		self.z = z * phi.cos() - x * phi.sin();
		self.x = z * phi.sin() + x * phi.cos();
	}

}
