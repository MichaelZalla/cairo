use std::{
    f32::consts::PI,
    fmt::{self, Display},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use super::vec::{vec3::Vec3, vec4::Vec4};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mat<T, const N: usize> {
    elements: [[T; N]; N],
}

impl<T: Copy + Display, const N: usize> Display for Mat<T, N> {
    fn fmt(&self, v: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result: Vec<String> = vec![String::from("(\n"); 1];

        for i in 0..N {
            result.push(String::from(" ("));
            for j in 0..N {
                let value = self.elements[i][j];
                result.push(String::from(format!("{}", value)));
                if j < N - 1 {
                    result.push(String::from(","));
                }
            }
            result.push(String::from(")"));
            if i < N - 1 {
                result.push(String::from(","));
            }
            result.push(String::from("\n"));
        }

        result.push(String::from(")\n"));

        let flat_result = result.join("");

        write!(v, "{}", flat_result)
    }
}

impl<T: MulAssign<f32>, const N: usize> MulAssign<f32> for Mat<T, N> {
    fn mul_assign(&mut self, rhs: f32) {
        for i in 0..N {
            for j in 0..N {
                self.elements[i][j] *= rhs;
            }
        }
    }
}

impl<T: Clone + MulAssign<f32>, const N: usize> Mul<f32> for Mat<T, N> {
    type Output = Mat<T, N>;
    fn mul(self, rhs: f32) -> Mat<T, N> {
        let mut result = self.clone();
        result *= rhs;
        result
    }
}

impl<T: DivAssign<f32>, const N: usize> DivAssign<f32> for Mat<T, N> {
    fn div_assign(&mut self, rhs: f32) {
        for i in 0..N {
            for j in 0..N {
                self.elements[i][j] /= rhs;
            }
        }
    }
}

impl<T: Clone + DivAssign<f32>, const N: usize> Div<f32> for Mat<T, N> {
    type Output = Mat<T, N>;
    fn div(self, rhs: f32) -> Mat<T, N> {
        let mut result = self.clone();
        result /= rhs;
        result
    }
}

impl<T: Default + Copy + Mul<Output = T> + AddAssign<T>, const N: usize> MulAssign<Self>
    for Mat<T, N>
{
    fn mul_assign(&mut self, rhs: Self) {
        let result = (*self) * rhs;
        for i in 0..N {
            for j in 0..N {
                self.elements[i][j] = result.elements[i][j];
            }
        }
    }
}

impl<T: Default + Copy + Mul<Output = T> + AddAssign<T>, const N: usize> Mul<Self> for Mat<T, N> {
    type Output = Mat<T, N>;

    fn mul(self, rhs: Self) -> Mat<T, N> {
        let mut result = Mat::<T, N>::new();

        for i in 0..N {
            for j in 0..N {
                let mut sum: T = T::default();

                for k in 0..N {
                    sum += self.elements[i][k] * rhs.elements[k][j];
                }

                result.elements[i][j] = sum;
            }
        }

        result
    }
}

impl<T: Default + Copy + Add<Output = T> + AddAssign<T>, const N: usize> AddAssign<Self>
    for Mat<T, N>
{
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            for j in 0..N {
                self.elements[i][j] += rhs.elements[i][j];
            }
        }
    }
}

impl<T: Default + Copy + Add<Output = T> + AddAssign<T>, const N: usize> Add<Self> for Mat<T, N> {
    type Output = Mat<T, N>;

    fn add(self, rhs: Self) -> Mat<T, N> {
        let mut result = Mat::<T, N>::new();

        for i in 0..N {
            for j in 0..N {
                result.elements[i][j] = self.elements[i][j] + rhs.elements[i][j];
            }
        }

        result
    }
}

impl<T: Default + Copy + Sub<Output = T> + SubAssign<T>, const N: usize> Sub<Self> for Mat<T, N> {
    type Output = Mat<T, N>;

    fn sub(self, rhs: Self) -> Mat<T, N> {
        let mut result = Mat::<T, N>::new();

        for i in 0..N {
            for j in 0..N {
                result.elements[i][j] = self.elements[i][j] - rhs.elements[i][j];
            }
        }

        result
    }
}

impl<T: Default + Copy, const N: usize> Mat<T, N> {
    pub fn new() -> Self {
        Mat {
            elements: [[T::default(); N]; N],
        }
    }

    pub fn new_from_elements(elements: [[T; N]; N]) -> Self {
        Mat { elements }
    }
}

impl<const N: usize> Mat<f32, N> {
    pub fn identity() -> Self {
        Self::scale([1.0; N])
    }

    pub fn scale(scale: [f32; N]) -> Self {
        let mut result = Mat::<f32, N>::new();

        for i in 0..N {
            result.elements[i][i] = scale[i];
        }

        result
    }
}

impl Mat<f32, 3> {
    pub fn rotation_x(theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Self {
            elements: [
                [1.0, 0.0, 0.0],
                [0.0, cos_theta, sin_theta],
                [0.0, -sin_theta, cos_theta],
            ],
        }
    }

    pub fn rotation_y(theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Self {
            elements: [
                [cos_theta, 0.0, -sin_theta],
                [0.0, 1.0, 0.0],
                [sin_theta, 0.0, cos_theta],
            ],
        }
    }

    pub fn rotation_z(theta: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        Self {
            elements: [
                [cos_theta, sin_theta, 0.0],
                [-sin_theta, cos_theta, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Mat<f32, 4> {
    pub fn from_mat3(mat: Mat3) -> Self {
        Self {
            elements: [
                [
                    mat.elements[0][0],
                    mat.elements[0][1],
                    mat.elements[0][2],
                    0.0,
                ],
                [
                    mat.elements[1][0],
                    mat.elements[1][1],
                    mat.elements[1][2],
                    0.0,
                ],
                [
                    mat.elements[2][0],
                    mat.elements[2][1],
                    mat.elements[2][2],
                    0.0,
                ],
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

    pub fn rotation_x(theta: f32) -> Self {
        Self::from_mat3(Mat3::rotation_x(theta))
    }

    pub fn rotation_y(theta: f32) -> Self {
        Self::from_mat3(Mat3::rotation_y(theta))
    }

    pub fn rotation_z(theta: f32) -> Self {
        Self::from_mat3(Mat3::rotation_z(theta))
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
                [0.0, 0.0, 2.0 / (far - near), 0.0],
                [
                    -(right + left) / (right - left),
                    -(top + bottom) / (top - bottom),
                    -(far + near) / (far - near),
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
                [0.0, 0.0, 1.0 / (2.0 / (far - near)), 0.0],
                [
                    (right + left) / (right - left),
                    (top + bottom) / (top - bottom),
                    (far + near) / (far - near),
                    1.0,
                ],
            ],
        }
    }

    pub fn perspective(width: f32, height: f32, near: f32, far: f32) -> Self {
        let (w, h, n, f) = (width, height, near, far);

        Self {
            elements: [
                [2.0 * n / w as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 * n / h as f32, 0.0, 0.0],
                [0.0, 0.0, f / (f - n), 1.0],
                [0.0, 0.0, (-n * f) / (f - n), 0.0],
            ],
        }
    }

    pub fn perspective_for_fov(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let fov_rad = fov * PI / 180.0;
        let width = 1.0 / (fov_rad / 2.0).tan();
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
        let fov_rad = fov * PI / 180.0;
        let width = 1.0 / (fov_rad / 2.0).tan();
        let height = width * aspect_ratio;

        let (w, h, n, f) = (width, height, near, far);

        Self {
            elements: [
                [1.0 / w, 0.0, 0.0, 0.0],
                [0.0, 1.0 / h, 0.0, 0.0],
                [0.0, 0.0, 1.0 / (f / (f - n)), 1.0],
                [0.0, 0.0, -((-n * f) / (f - n)), 0.0],
            ],
        }
    }

    pub fn look_at(position: Vec3, forward: Vec3, right: Vec3, up: Vec3) -> Mat4 {
        let (p, f, r, u) = (position, forward, right, up);

        let rotation_transposed = Mat4::new_from_elements([
            // Row-major ordering
            [r.x, u.x, f.x, 0.0],
            [r.y, u.y, f.y, 0.0],
            [r.z, u.z, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let translation_negated = Mat4::new_from_elements([
            // Row-major ordering
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-p.x, -p.y, -p.z, 1.0],
        ]);

        translation_negated * rotation_transposed
    }

    pub fn look_at_inverse(position: Vec3, forward: Vec3, right: Vec3, up: Vec3) -> Mat4 {
        let (p, f, r, u) = (position, forward, right, up);

        let rotation = Mat4::new_from_elements([
            // Row-major ordering
            [r.x, r.y, r.z, 0.0],
            [u.x, u.y, u.z, 0.0],
            [f.x, f.y, f.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let translation = Mat4::new_from_elements([
            // Row-major ordering
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [p.x, p.y, p.z, 1.0],
        ]);

        rotation * translation
    }

    pub fn transposed(&self) -> Self {
        let mut result: Self = self.clone();

        for i in 0..self.elements.len() {
            for j in 0..self.elements[0].len() {
                result.elements[i][j] = self.elements[j][i];
            }
        }

        result
    }
}

pub type Mat3 = Mat<f32, 3>;

impl Default for Mat3 {
    fn default() -> Self {
        Self::identity()
    }
}

impl MulAssign<Mat3> for Vec3 {
    fn mul_assign(&mut self, rhs: Mat3) {
        let result = self.clone() * rhs;

        self.x = result.x;
        self.y = result.y;
        self.z = result.z;
    }
}

impl Mul<Mat3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Mat3) -> Self {
        Vec3 {
            x: (self.x * rhs.elements[0][0]
                + self.y * rhs.elements[1][0]
                + self.z * rhs.elements[2][0]),
            y: (self.x * rhs.elements[0][1]
                + self.y * rhs.elements[1][1]
                + self.z * rhs.elements[2][1]),
            z: (self.x * rhs.elements[0][2]
                + self.y * rhs.elements[1][2]
                + self.z * rhs.elements[2][2]),
        }
    }
}

pub type Mat4 = Mat<f32, 4>;

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl MulAssign<Mat4> for Vec4 {
    fn mul_assign(&mut self, rhs: Mat4) {
        let result = self.clone() * rhs;

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
