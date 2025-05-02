use std::{cmp, fmt, ops, str::FromStr};

use rand::rngs::ThreadRng;

use rand_distr::{Distribution, Uniform};

use serde::{
    ser::{Serialize, SerializeStruct, Serializer},
    Deserialize, Deserializer,
};

use serde_tuple::{Deserialize_tuple, Serialize_tuple};

use super::vec2::Vec2;

#[derive(Default, Debug, Copy, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone)]
pub union Vec3A {
    pub v: Vec3,
    pub a: [f32; 3],
}

impl Serialize for Vec3A {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Vec3A", 3)?;

        unsafe {
            state.serialize_field("x", &self.v.x)?;
            state.serialize_field("y", &self.v.y)?;
            state.serialize_field("z", &self.v.z)?;
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for Vec3A {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Default::default())
    }
}

impl Default for Vec3A {
    fn default() -> Self {
        let v = Vec3::default();

        Self { v }
    }
}

impl fmt::Debug for Vec3A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { write!(f, "{}", self.v) }
    }
}

impl From<Vec3> for Vec3A {
    fn from(v: Vec3) -> Self {
        Self { v }
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.*},{:.*},{:.*})", 2, self.x, 2, self.y, 2, self.z)
    }
}

impl fmt::Display for Vec3A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.v })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseVec3Error;

impl FromStr for Vec3 {
    type Err = ParseVec3Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let channels: Vec<String> = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .map(|s| s.splitn(4, ','))
            .map(|s| s.map(|c| c.to_string()))
            .map(|s| s.collect())
            .ok_or(ParseVec3Error)?;

        debug_assert!(channels.len() == 3);

        let x = channels[0].parse::<f32>().map_err(|_| ParseVec3Error)?;
        let y = channels[1].parse::<f32>().map_err(|_| ParseVec3Error)?;
        let z = channels[2].parse::<f32>().map_err(|_| ParseVec3Error)?;

        Ok(Vec3 { x, y, z })
    }
}

impl Vec3 {
    pub fn uniform(rng: &mut ThreadRng, uniform: &Uniform<f32>) -> Self {
        Self {
            x: uniform.sample(rng),
            y: uniform.sample(rng),
            z: uniform.sample(rng),
        }
    }

    pub fn from_x_y(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
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

macro_rules! derive_op {
    ($trait:ident, $method:ident, $operator:tt) => {
        impl ops::$trait for Vec3 {
            type Output = Vec3;
            fn $method(self, rhs: Vec3) -> Vec3 {
                Vec3 {
                    x: self.x $operator rhs.x,
                    y: self.y $operator rhs.y,
                    z: self.z $operator rhs.z,
                }
            }
        }

        impl ops::$trait<&Vec3> for Vec3 {
            type Output = Vec3;
            fn $method(self, rhs: &Vec3) -> Vec3 {
                Vec3 {
                    x: self.x $operator rhs.x,
                    y: self.y $operator rhs.y,
                    z: self.z $operator rhs.z,
                }
            }
        }

        impl ops::$trait<Vec3> for &Vec3 {
            type Output = Vec3;
            fn $method(self, rhs: Vec3) -> Vec3 {
                Vec3 {
                    x: self.x $operator rhs.x,
                    y: self.y $operator rhs.y,
                    z: self.z $operator rhs.z,
                }
            }
        }

        impl ops::$trait<&Vec3> for &Vec3 {
            type Output = Vec3;
            fn $method(self, rhs: &Vec3) -> Vec3 {
                Vec3 {
                    x: self.x $operator rhs.x,
                    y: self.y $operator rhs.y,
                    z: self.z $operator rhs.z,
                }
            }
        }
    };
}

derive_op!(Add, add, +);
derive_op!(Sub, sub, -);
derive_op!(Mul, mul, *);
derive_op!(Div, div, /);

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

impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
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

impl ops::DivAssign for Vec3 {
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

impl ops::MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Vec3 {
    pub fn min(&self, rhs: &Vec3) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
            z: self.z.min(rhs.z),
        }
    }

    pub fn max(&self, rhs: &Vec3) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
            z: self.z.max(rhs.z),
        }
    }

    pub fn extent(points: &[Vec3]) -> (Vec3, Vec3) {
        let mut min = MAX;
        let mut max = MIN;

        for v in points {
            min = min.min(v);
            max = max.max(v);
        }

        (min, max)
    }

    pub fn mag_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn mag(self) -> f32 {
        self.mag_squared().sqrt()
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

    pub fn angle_radians(a: &Self, b: &Self, c: &Self) -> f32 {
        // See: https://math.stackexchange.com/a/3427603/155265
        // See: https://gamedev.stackexchange.com/a/203308

        let b_a = a - b;
        let b_c = c - b;

        let b_a_mag = b_a.mag();
        let b_c_mag = b_c.mag();

        (b_a.dot(b_c) / (b_a_mag * b_c_mag)).acos()
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

    pub fn reciprocal(&self) -> Self {
        Self {
            x: 1.0 / self.x,
            y: 1.0 / self.y,
            z: 1.0 / self.z,
        }
    }

    pub fn half_area_of_extent(&self) -> f32 {
        self.x * self.y + self.y * self.z + self.x * self.z
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

    pub fn ndc_to_uv(&self) -> Vec2 {
        Vec2 {
            x: self.x * 0.5 + 0.5,
            y: self.y * 0.5 + 0.5,
            z: 0.0,
        }
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

    pub fn luminance(&self) -> f32 {
        // See: https://en.wikipedia.org/wiki/Rec._709

        static LUMA_COEFFICIENTS: Vec3 = Vec3 {
            x: 0.2126,
            y: 0.7152,
            z: 0.0722,
        };

        self.dot(LUMA_COEFFICIENTS)
    }

    pub fn with_luminance(&self, luminance: f32) -> Self {
        let (from, to) = (self.luminance(), luminance);

        *self * to / from
    }

    pub fn clamp_max(&self, upper_limit: f32) -> Self {
        Self {
            x: self.x.min(upper_limit),
            y: self.y.min(upper_limit),
            z: self.z.min(upper_limit),
        }
    }

    pub fn clamp_min(&self, lower_limit: f32) -> Self {
        Self {
            x: self.x.max(lower_limit),
            y: self.y.max(lower_limit),
            z: self.z.max(lower_limit),
        }
    }

    pub fn clamp(&self, min: f32, max: f32) -> Self {
        self.clamp_min(min).clamp_max(max)
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
