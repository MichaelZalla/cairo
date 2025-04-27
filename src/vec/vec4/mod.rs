use std::cmp;
use std::fmt;
use std::ops;

use serde_tuple::{Deserialize_tuple, Serialize_tuple};

use super::vec2::Vec2;
use super::vec3;
use super::vec3::Vec3;

#[derive(Debug, Copy, Clone, Default, Serialize_tuple, Deserialize_tuple)]
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
    pub const fn new(v: Vec3, w: f32) -> Self {
        Vec4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
    }

    pub const fn vector(v: Vec3) -> Self {
        Self::new(v, 0.0)
    }

    pub const fn position(v: Vec3) -> Self {
        Self::new(v, 1.0)
    }
}

impl cmp::PartialEq for Vec4 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
    }
}

impl ops::Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl ops::AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Vec4) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl ops::Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl ops::SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Vec4) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl ops::Mul for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 {
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
        Vec4 {
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
        Vec4 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl ops::DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) {
        self.x = self.x / rhs;
        self.y = self.y / rhs;
        self.z = self.z / rhs;
        self.w = self.w / rhs;
    }
}

impl ops::Div for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
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

impl ops::MulAssign for Vec4 {
    fn mul_assign(&mut self, rhs: Vec4) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

impl Vec4 {
    pub fn to_vec3(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn max(&self) -> f32 {
        self.x.max(self.y.max(self.z))
    }

    pub fn abs(self) -> Self {
        Vec4 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
            w: self.w.abs(),
        }
    }

    pub fn mag(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn dot(self, rhs: Vec4) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
            w: self.w,
        }
    }

    pub fn as_normal(self) -> Vec4 {
        let mag = self.mag();
        Vec4 {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
            w: self.w,
        }
    }

    pub fn ndc_to_uv(&self) -> Vec2 {
        Vec2 {
            x: 0.5 + self.x / 2.0,
            y: 0.5 + self.y / 2.0,
            z: 0.0,
        }
    }
}

pub static UP: Vec4 = Vec4::vector(vec3::UP);

pub static RIGHT: Vec4 = Vec4::vector(vec3::RIGHT);

pub static FORWARD: Vec4 = Vec4::vector(vec3::FORWARD);
