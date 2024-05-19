use std::{
    f32::consts::PI,
    fmt::{self, Display},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub},
};

use serde::{Deserialize, Serialize};

use super::vec::{vec3::Vec3, vec4::Vec4};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mat4 {
    elements: [[f32; 4]; 4],
}

impl Display for Mat4 {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result: Vec<String> = vec![String::from("(\n"); 1];

        for i in 0..4 {
            result.push(String::from(" ("));
            for j in 0..4 {
                let value = self.elements[i][j];
                result.push(format!("{}", value));
                if j < 4 - 1 {
                    result.push(String::from(","));
                }
            }
            result.push(String::from(")"));
            if i < 4 - 1 {
                result.push(String::from(","));
            }
            result.push(String::from("\n"));
        }

        result.push(String::from(")\n"));

        let flat_result = result.join("");

        write!(v, "{}", flat_result)
    }
}

impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, rhs: f32) {
        for i in 0..4 {
            for j in 0..4 {
                self.elements[i][j] *= rhs;
            }
        }
    }
}

impl Mul<f32> for Mat4 {
    type Output = Mat4;
    fn mul(self, rhs: f32) -> Mat4 {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl DivAssign<f32> for Mat4 {
    fn div_assign(&mut self, rhs: f32) {
        for i in 0..4 {
            for j in 0..4 {
                self.elements[i][j] /= rhs;
            }
        }
    }
}

impl Div<f32> for Mat4 {
    type Output = Mat4;
    fn div(self, rhs: f32) -> Mat4 {
        let mut result = self;
        result /= rhs;
        result
    }
}

impl MulAssign<Self> for Mat4 {
    fn mul_assign(&mut self, rhs: Self) {
        let result = (*self) * rhs;
        for i in 0..4 {
            for j in 0..4 {
                self.elements[i][j] = result.elements[i][j];
            }
        }
    }
}

impl Mul<Self> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Mat4 {
        let mut result = Mat4::new();

        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;

                for k in 0..4 {
                    sum += self.elements[i][k] * rhs.elements[k][j];
                }

                result.elements[i][j] = sum;
            }
        }

        result
    }
}

impl AddAssign<Self> for Mat4 {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..4 {
            for j in 0..4 {
                self.elements[i][j] += rhs.elements[i][j];
            }
        }
    }
}

impl Add<Self> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: Self) -> Mat4 {
        let mut result = Mat4::new();

        for i in 0..4 {
            for j in 0..4 {
                result.elements[i][j] = self.elements[i][j] + rhs.elements[i][j];
            }
        }

        result
    }
}

impl Sub<Self> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: Self) -> Mat4 {
        let mut result = Mat4::new();

        for i in 0..4 {
            for j in 0..4 {
                result.elements[i][j] = self.elements[i][j] - rhs.elements[i][j];
            }
        }

        result
    }
}

impl Mat4 {
    pub fn new() -> Self {
        Self {
            elements: [[Default::default(); 4]; 4],
        }
    }

    pub fn new_from_elements(elements: [[f32; 4]; 4]) -> Self {
        Self { elements }
    }

    pub fn identity() -> Self {
        Self::scale([1.0; 4])
    }

    pub fn scale(scale: [f32; 4]) -> Self {
        let mut result = Mat4::new();

        for i in 0..4 {
            result.elements[i][i] = scale[i];
        }

        result
    }

    pub fn rotation_x(theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Self {
            elements: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos_theta, sin_theta, 0.0],
                [0.0, -sin_theta, cos_theta, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y(theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Self {
            elements: [
                [cos_theta, 0.0, -sin_theta, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [sin_theta, 0.0, cos_theta, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_z(theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Self {
            elements: [
                [cos_theta, sin_theta, 0.0, 0.0],
                [-sin_theta, cos_theta, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn tbn(t: Vec3, b: Vec3, n: Vec3) -> Self {
        Self {
            elements: [
                [t.x, t.y, t.z, 0.0],
                [b.x, b.y, b.z, 0.0],
                [n.x, n.y, n.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn translation(v: Vec3) -> Self {
        Self {
            elements: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [v.x, v.y, v.z, 1.0],
            ],
        }
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            // Row-major ordering
            elements: [
                [2.0 / (right - left), 0.0, 0.0, 0.0],
                [0.0, 2.0 / (top - bottom), 0.0, 0.0],
                [0.0, 0.0, 1.0 / (far - near), 0.0],
                [
                    -(right + left) / (right - left),
                    -(top + bottom) / (top - bottom),
                    -near,
                    1.0,
                ],
            ],
        }
    }

    pub fn orthographic_inverse(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            // Row-major ordering
            elements: [
                [1.0 / (2.0 / (right - left)), 0.0, 0.0, 0.0],
                [0.0, 1.0 / (2.0 / (top - bottom)), 0.0, 0.0],
                [0.0, 0.0, far - near, 0.0],
                [
                    (right + left) / (right - left),
                    (top + bottom) / (top - bottom),
                    near,
                    1.0,
                ],
            ],
        }
    }

    pub fn perspective(width: f32, height: f32, near: f32, far: f32) -> Self {
        let (w, h, n, f) = (width, height, near, far);

        Self {
            elements: [
                [2.0 * n / w, 0.0, 0.0, 0.0],
                [0.0, 2.0 * n / h, 0.0, 0.0],
                [0.0, 0.0, f / (f - n), 1.0],
                [0.0, 0.0, (-n * f) / (f - n), 0.0],
            ],
        }
    }

    pub fn get_width_for_fov(fov: f32) -> f32 {
        let fov_rad = fov * PI / 180.0;

        1.0 / (fov_rad / 2.0).tan()
    }

    pub fn perspective_for_fov(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let width = Self::get_width_for_fov(fov);
        let height = width * aspect_ratio;

        let (w, h, n, f) = (width, height, near, far);

        Self {
            elements: [
                [w, 0.0, 0.0, 0.0],
                [0.0, h, 0.0, 0.0],
                [0.0, 0.0, f / (f - n), 1.0],
                [0.0, 0.0, (-n * f) / (f - n), 0.0],
            ],
        }
    }

    pub fn perspective_inverse_for_fov(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let width = Self::get_width_for_fov(fov);
        let height = width * aspect_ratio;

        let (w, h, n, f) = (width, height, near, far);

        Self {
            elements: [
                [1.0 / w, 0.0, 0.0, 0.0],
                [0.0, 1.0 / h, 0.0, 0.0],
                [0.0, 0.0, 1.0 / (f / (f - n)), 1.0],
                [0.0, 0.0, (n * f) / (f - n), 0.0],
            ],
        }
    }

    pub fn look_at(position: Vec3, forward: Vec3, right: Vec3, up: Vec3) -> Mat4 {
        let (f, r, u) = (forward, right, up);

        let rotation_transposed = Mat4::new_from_elements([
            // Row-major ordering
            [r.x, u.x, f.x, 0.0],
            [r.y, u.y, f.y, 0.0],
            [r.z, u.z, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let translation_negated = Mat4::translation(-position);

        translation_negated * rotation_transposed
    }

    pub fn look_at_inverse(position: Vec3, forward: Vec3, right: Vec3, up: Vec3) -> Mat4 {
        let (f, r, u) = (forward, right, up);

        let rotation = Mat4::new_from_elements([
            // Row-major ordering
            [r.x, r.y, r.z, 0.0],
            [u.x, u.y, u.z, 0.0],
            [f.x, f.y, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        rotation * Mat4::translation(position)
    }

    pub fn transposed(&self) -> Self {
        let mut result: Self = *self;

        for i in 0..self.elements.len() {
            for j in 0..self.elements[0].len() {
                result.elements[i][j] = self.elements[j][i];
            }
        }

        result
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl MulAssign<Mat4> for Vec4 {
    fn mul_assign(&mut self, rhs: Mat4) {
        let result = *self * rhs;

        self.x = result.x;
        self.y = result.y;
        self.z = result.z;
        self.w = result.w;
    }
}

impl Mul<Mat4> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: Mat4) -> Self {
        Vec4 {
            x: (self.x * rhs.elements[0][0]
                + self.y * rhs.elements[1][0]
                + self.z * rhs.elements[2][0]
                + self.w * rhs.elements[3][0]),
            y: (self.x * rhs.elements[0][1]
                + self.y * rhs.elements[1][1]
                + self.z * rhs.elements[2][1]
                + self.w * rhs.elements[3][1]),
            z: (self.x * rhs.elements[0][2]
                + self.y * rhs.elements[1][2]
                + self.z * rhs.elements[2][2]
                + self.w * rhs.elements[3][2]),
            w: (self.x * rhs.elements[0][3]
                + self.y * rhs.elements[1][3]
                + self.z * rhs.elements[2][3]
                + self.w * rhs.elements[3][3]),
        }
    }
}
