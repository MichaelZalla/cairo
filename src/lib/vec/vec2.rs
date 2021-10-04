use std::fmt;
use std::ops;

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}

impl fmt::Display for Vec2 {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(v, "({}, {})", self.x, self.y)
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2{
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
    }
}

impl ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) -> () {
		self.x += rhs.x;
		self.y += rhs.y;
    }
}

impl ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2{
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2{
			x: self.x * rhs,
			y: self.y * rhs,
		}
    }
}

impl ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2{
			x: self.x * rhs.x,
			y: self.y * rhs.y,
		}
    }
}

impl ops::MulAssign<Vec2> for Vec2 {
    fn mul_assign(&mut self, rhs: Vec2) {
		self.x *= rhs.x;
		self.y *= rhs.y;
	}
}

// impl ops::Div<f32> for Vec2 {
//     type Output = Vec2;
//     fn div(self, rhs: f32) -> Vec2 {
//         Vec2{
// 			x: self.x / rhs,
// 			y: self.y / rhs,
// 		}
//     }
// }

// impl ops::Div<Vec2> for Vec2 {
//     type Output = Vec2;
//     fn div(self, rhs: Vec2) -> Vec2 {
//         Vec2{
// 			x: self.x / rhs.x,
// 			y: self.y / rhs.y,
// 		}
//     }
// }

impl Vec2 {

	fn len(self) -> f32 {
		return ((self.x.powi(2) + self.y.powi(2)) / 2.0).sqrt();
	}

	fn dot(self, rhs: Vec2) -> f32 {
		return self.x * rhs.x + self.y * rhs.y;
	}

	fn as_normal(self) -> Vec2 {
		let len = self.len();
		Vec2{
			x: self.x / len,
			y: self.y / len,
		}
	}

	fn normalize(&mut self) -> () {
		let len = self.len();
		self.x /= len;
		self.y /= len;
	}

	fn rotate(&mut self, phi: f32) -> () {
		let (x, y, phi_cos, phi_sin) = (self.x, self.y, phi.cos(), phi.sin());
		self.x = x * phi_cos - y * phi_sin;
		self.y = x * phi_sin + y * phi_cos;
	}

}
