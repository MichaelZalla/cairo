use std::fmt;
use std::ops;

#[derive(Debug, Copy, Clone, Default)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
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
			z: self.z,
		}
    }
}

impl ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
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
			z: self.z,
		}
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2{
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z,
		}
    }
}

impl ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2{
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z,
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