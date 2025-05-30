use std::fmt;
use std::ops;

use serde_tuple::Deserialize_tuple;
use serde_tuple::Serialize_tuple;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl fmt::Display for Vec2 {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(v, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Vec2 {
    pub fn offset(&self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            z: self.z,
        }
    }
}
