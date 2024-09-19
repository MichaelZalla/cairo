use std::ops;

use cairo::{
    matrix::Mat4,
    vec::vec3::{self, Vec3},
};

#[derive(Default, Debug, Copy, Clone)]
pub struct Quaternion {
    pub s: f32,
    pub u: Vec3,
    mat: Mat4,
}

impl ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let (a, b) = (self, rhs);

        // See: https://stackoverflow.com/a/19956940/1623811

        let s = a.s * b.s - a.u.x * b.u.x - a.u.y * b.u.y - a.u.z * b.u.z;

        let x = a.s * b.u.x + a.u.x * b.s + a.u.y * b.u.z - a.u.z * b.u.y;
        let y = a.s * b.u.y - a.u.x * b.u.z + a.u.y * b.s + a.u.z * b.u.x;
        let z = a.s * b.u.z + a.u.x * b.u.y - a.u.y * b.u.x + a.u.z * b.s;

        let mat = quaternion_to_mat4(s, x, y, z);

        Self {
            s,
            u: Vec3 { x, y, z },
            mat,
        }
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

impl Quaternion {
    pub fn new(axis_normal: Vec3, theta: f32) -> Self {
        let theta_over_2 = theta / 2.0;

        let s = theta_over_2.cos();
        let u = axis_normal * theta_over_2.sin();
        let mat = quaternion_to_mat4(s, u.x, u.y, u.z);

        Self { s, u, mat }
    }

    pub fn new_2d(theta: f32) -> Self {
        Self::new(-vec3::FORWARD, theta)
    }

    pub fn mat(&self) -> &Mat4 {
        &self.mat
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
