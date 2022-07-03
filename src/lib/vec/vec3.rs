use std::cmp;
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

impl cmp::PartialEq for Vec3 {
	fn eq(&self, other: &Self) -> bool {
        self.x == other.x &&
        self.y == other.y &&
        self.z == other.z
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

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3{
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
    }
}

impl ops::SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
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

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3{
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
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

	fn mag(self) -> f32 {
		return ((self.x.powi(2) + self.y.powi(2) + self.z.powi(2)) / 2.0).sqrt();
	}

	pub fn dot(self, rhs: Vec3) -> f32 {
		// return self.mag() * rhs.mag() * theta.cos();
		return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
	}

	pub fn cross(self, rhs: Vec3) -> Vec3 {
		return Vec3 {
			x: self.y * rhs.z - self.z * rhs.y,
			y: self.z * rhs.x - self.x * rhs.z,
			z: self.x * rhs.y - self.y * rhs.x,
		};
	}

	pub fn as_normal(self) -> Vec3 {
		let mag = self.mag();
		Vec3{
			x: self.x / mag,
			y: self.y / mag,
			z: self.z / mag,
		}
	}

	// fn normalize(&mut self) -> () {
	// 	let mag = self.mag();
	// 	self.x /= mag;
	// 	self.y /= mag;
	// 	self.z /= mag;
	// }

	pub fn rotate_along_z(&mut self, phi: f32) -> () {
		let (x, y, phi_cos, phi_sin) = (self.x, self.y, phi.cos(), phi.sin());
		self.x = x * phi_cos - y * phi_sin;
		self.y = x * phi_sin + y * phi_cos;
	}

	pub fn rotate_along_x(&mut self, phi: f32) -> () {
		// X-axis rotation looks like Z-axis rotation if replace:
		// - X axis with Y axis
		// - Y axis with Z axis
		// - Z axis with X axis
		let (y, z, phi_cos, phi_sin) = (self.y, self.z, phi.cos(), phi.sin());
		self.y = y * phi_cos - z * phi_sin;
		self.z = y * phi_sin + z * phi_cos;
	}

	pub fn rotate_along_y(&mut self, phi: f32) -> () {
		// Y-axis rotation looks like Z-axis rotation if replace:
		// - X axis with Z axis
		// - Y axis with X axis
		// - Zaxis with Y axis
		let (z, x, phi_cos, phi_sin) = (self.z, self.x, phi.cos(), phi.sin());
		self.z = z * phi_cos - x * phi_sin;
		self.x = z * phi_sin + x * phi_cos;
	}

}
