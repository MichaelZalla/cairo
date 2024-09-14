use std::cmp;
use std::fmt;
use std::ops;

use serde_tuple::Deserialize_tuple;
use serde_tuple::Serialize_tuple;

use crate::animation::lerp;

#[derive(Debug, Copy, Clone, Default, Serialize_tuple, Deserialize_tuple)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl fmt::Display for Vec3 {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(v, "({:.*}, {:.*}, {:.*})", 2, self.x, 2, self.y, 2, self.z)
    }
}

impl Vec3 {
    pub fn new() -> Self {
        Default::default()
    }

    pub const fn ones() -> Self {
        Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
}

impl cmp::PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Add<f32> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
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
        Vec3 {
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
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Div<Vec3> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl ops::DivAssign<Vec3> for Vec3 {
    fn div_assign(&mut self, rhs: Vec3) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
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
    pub fn extent(points: &[Vec3]) -> (Vec3, Vec3) {
        let mut min = MAX;
        let mut max = MIN;

        for v in points {
            if v.x < min.x {
                min.x = v.x;
            } else if v.x > max.x {
                max.x = v.x;
            }

            if v.y < min.y {
                min.y = v.y;
            } else if v.y > max.y {
                max.y = v.y;
            }

            if v.z < min.z {
                min.z = v.z;
            } else if v.z > max.z {
                max.z = v.z;
            }
        }

        (min, max)
    }

    pub fn mag(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn is_zero(self) -> bool {
        self.x.abs() < f32::EPSILON && self.y.abs() < f32::EPSILON && self.z.abs() < f32::EPSILON
    }

    pub fn as_normal(self) -> Self {
        let mag = self.mag();

        Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    pub fn reflect(self, rhs: Self) -> Self {
        // Project the incoming ray forward through the fragment/surface
        let absorbed_ray = self;

        // Project the incoming light ray onto the surface normal (i.e.,
        // scaling the normal up or down)
        let w = rhs * self.dot(rhs);

        // Combine the absorbed ray with the scaled normal to find the
        // reflected ray vector.
        let u = w * 2.0;

        u - absorbed_ray
    }

    pub fn interpolate(start: Self, end: Self, alpha: f32) -> Self {
        lerp(start, end, alpha)
    }

    pub fn srgb_to_linear(&mut self) {
        self.x = self.x * self.x;
        self.y = self.y * self.y;
        self.z = self.z * self.z;
    }

    pub fn linear_to_srgb(&mut self) {
        self.x = self.x.sqrt();
        self.y = self.y.sqrt();
        self.z = self.z.sqrt();
    }

    pub fn tone_map_exposure(&self, exposure: f32) -> Self {
        Self::ones()
            - Self {
                x: (-self.x * exposure).exp(),
                y: (-self.y * exposure).exp(),
                z: (-self.z * exposure).exp(),
            }
    }
}

pub static MIN: Vec3 = Vec3 {
    x: f32::MIN,
    y: f32::MIN,
    z: f32::MIN,
};

pub static MAX: Vec3 = Vec3 {
    x: f32::MAX,
    y: f32::MAX,
    z: f32::MAX,
};

pub static ONES: Vec3 = Vec3::ones();

pub static UP: Vec3 = Vec3 {
    x: -0.0,
    y: 1.0,
    z: -0.0,
};

pub static RIGHT: Vec3 = Vec3 {
    x: 1.0,
    y: 0.0,
    z: 0.0,
};

pub static FORWARD: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};
