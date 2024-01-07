use std::cmp;
use std::fmt;
use std::ops;

#[derive(Debug, Copy, Clone, Default)]
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

impl Vec3 {
    pub fn new() -> Self {
        Default::default()
    }
}

impl cmp::PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
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
    pub fn mag(self) -> f32 {
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
        Vec3 {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    pub fn hadamard(&mut self, rhs: Vec3) {
        *self *= rhs;
    }

    pub fn get_hadamard(&self, v: Vec3) -> Vec3 {
        let mut result = self.clone();

        result.hadamard(v);

        return result;
    }

    pub fn saturate(&mut self) -> &Self {
        self.x = self.x.max(0.0).min(1.0);
        self.y = self.y.max(0.0).min(1.0);
        self.z = self.z.max(0.0).min(1.0);

        return self;
    }

    pub fn interpolate(start: Self, end: Self, alpha: f32) -> Self {
        return start + (end - start) * alpha;
    }
}

pub const UP: Vec3 = Vec3 {
    x: -0.0,
    y: 1.0,
    z: -0.0,
};

pub const LEFT: Vec3 = Vec3 {
    x: -1.0,
    y: 0.0,
    z: 0.0,
};

pub const FORWARD: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};
