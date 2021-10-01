use std::ops;
use std::fmt;

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

// impl ops::Sub<Vec2> for Vec2 {
//     type Output = Vec2;
//     fn sub(self, rhs: Vec2) -> Vec2 {
//         Vec2{
// 			x: self.x - rhs.x,
// 			y: self.y - rhs.y,
// 		}
//     }
// }

// impl ops::Mul<f32> for Vec2 {
//     type Output = Vec2;
//     fn mul(self, rhs: f32) -> Vec2 {
//         Vec2{
// 			x: self.x * rhs,
// 			y: self.y * rhs,
// 		}
//     }
// }

impl ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2{
			x: self.x * rhs.x,
			y: self.y * rhs.y,
		}
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

	// fn dot(&mut self, from_vec: Vec2) -> () {
	// 	self.x  += from_vec.x;
	// 	self.y  += from_vec.y;
	// }

}

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

// impl ops::AddEquals<Vec3> for Vec3 {
//     type Output = Vec3;
//     fn addEquals(self, rhs: Vec3) -> () {
// 		self.x += rhs.x;
// 		self.y += rhs.y;
// 		self.z += rhs.z;
//     }
// }

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

pub struct Mesh {
	pub v: Vec<Vec3>,
	pub f: Vec<(usize, usize, usize)>,
}
