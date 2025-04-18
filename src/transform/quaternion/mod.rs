use std::{fmt, ops};

use serde::{Deserialize, Serialize};

use crate::{
    matrix::Mat4,
    vec::vec3::{self, Vec3},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub s: f32,
    pub u: Vec3,
    mat: Mat4,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::new_2d(0.0)
    }
}

impl fmt::Display for Quaternion {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(v, "(s={}, u={})", self.s, self.u)
    }
}

impl ops::AddAssign for Quaternion {
    fn add_assign(&mut self, rhs: Self) {
        self.s += rhs.s;
        self.u += rhs.u;
    }
}

impl ops::Add for Quaternion {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let (a, b) = (self, rhs);

        //
        // See: https://www.ashwinnarayan.com/post/how-to-integrate-quaternions/#multiplication
        //
        // a * b = [
        //   (a.s * b.s - a.u.dot(b.u)),
        //   a.s * b.u + b.s * a.u + a.u.cross(b.u)
        // ]
        //

        let s = a.s * b.s - a.u.dot(b.u);
        let u = (b.u * a.s) + (a.u * b.s) + a.u.cross(b.u);

        let mat = quaternion_to_mat4(s, u.x, u.y, u.z);

        Self { s, u, mat }
    }
}

impl ops::MulAssign for Quaternion {
    fn mul_assign(&mut self, rhs: Self) {
        let product = *self * rhs;

        self.s = product.s;
        self.u = product.u;
        self.mat = product.mat;
    }
}

impl ops::Mul<f32> for Quaternion {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        let s = self.s * scalar;
        let u = self.u * scalar;
        let mat = quaternion_to_mat4(s, u.x, u.y, u.z);

        Self { s, u, mat }
    }
}

impl ops::MulAssign<f32> for Quaternion {
    fn mul_assign(&mut self, rhs: f32) {
        let product = *self * rhs;

        self.s = product.s;
        self.u = product.u;
        self.mat = product.mat;
    }
}

impl Quaternion {
    pub fn new(axis_normal: Vec3, theta: f32) -> Self {
        let theta_over_2 = theta / 2.0;

        let s = theta_over_2.cos();
        let u = axis_normal * theta_over_2.sin();
        let mat = Default::default();

        let mut result = Self { s, u, mat };

        result.renormalize();

        result
    }

    pub fn from_raw(s: f32, u: Vec3) -> Self {
        let mut result = Self {
            s,
            u,
            mat: Default::default(),
        };

        result.recompute_derived_state();

        result
    }

    pub fn new_2d(theta: f32) -> Self {
        Self::new(-vec3::FORWARD, theta)
    }

    #[allow(unused)]
    pub fn conjugate(&self) -> Self {
        let s = self.s;
        let u = -self.u;
        let mat = quaternion_to_mat4(s, u.x, u.y, u.z);

        Self { s, u, mat }
    }

    #[allow(unused)]
    pub fn inverse(&self) -> Self {
        self.conjugate() * (1.0 / self.mag_squared())
    }

    pub fn mat(&self) -> &Mat4 {
        &self.mat
    }

    pub fn theta(&self) -> f32 {
        // See: https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
        //
        // "…due to the periodic nature of sine and cosine, rotation angles
        // differing precisely by the natural period will be encoded into
        // identical quaternions and recovered angles in radians will be limited
        // to [0, 2*PI]."

        let cos_theta_over_2 = self.s;
        let sin_theta_over_2 = self.u.mag();

        let theta_over_2 = sin_theta_over_2.atan2(cos_theta_over_2);

        theta_over_2 * 2.0
    }

    pub fn renormalize(&mut self) {
        let mag = self.mag();

        self.s /= mag;

        self.u /= mag;

        self.recompute_derived_state();
    }

    fn recompute_derived_state(&mut self) {
        self.mat = quaternion_to_mat4(self.s, self.u.x, self.u.y, self.u.z);
    }

    fn mag_squared(&self) -> f32 {
        self.s * self.s + self.u.mag_squared()
    }

    fn mag(&self) -> f32 {
        self.mag_squared().sqrt()
    }
}

fn quaternion_to_mat4(s: f32, x: f32, y: f32, z: f32) -> Mat4 {
    let (x_x, x_y, x_z) = (x * x, x * y, x * z);
    let (y_y, y_z) = (y * y, y * z);
    let z_z = z * z;
    let (s_x, s_y, s_z) = (s * x, s * y, s * z);

    Mat4::new_from_elements([
        [
            1.0 - 2.0 * y_y - 2.0 * z_z,
            2.0 * x_y - 2.0 * s_z,
            2.0 * x_z + 2.0 * s_y,
            0.0,
        ],
        [
            2.0 * x_y + 2.0 * s_z,
            1.0 - 2.0 * x_x - 2.0 * z_z,
            2.0 * y_z - 2.0 * s_x,
            0.0,
        ],
        [
            2.0 * x_z - 2.0 * s_y,
            2.0 * y_z + 2.0 * s_x,
            1.0 - 2.0 * x_x - 2.0 * y_y,
            0.0,
        ],
        [0.0, 0.0, 0.0, 1.0],
    ])
}
