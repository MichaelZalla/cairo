use std::{fmt, ops};

use super::vec::{vec3::Vec3, vec4::Vec4};

#[derive(Debug, Copy, Clone)]
pub struct Mat<T, const N: usize> {
	elements: [[T; N]; N],
}

impl<T: std::marker::Copy + std::fmt::Display, const N: usize> fmt::Display for Mat<T, N> {
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

impl<T: std::ops::MulAssign<f32>, const N: usize> ops::MulAssign<f32> for Mat<T,N> {
	fn mul_assign(&mut self, rhs: f32) {
		for i in 0..N {
			for j in 0..N {
				self.elements[i][j] *= rhs;
			}
		}
	}
}

impl<T: std::clone::Clone + std::ops::MulAssign<f32>, const N: usize> ops::Mul<f32> for Mat<T,N> {
	type Output = Mat<T,N>;
	fn mul(self, rhs: f32) -> Mat<T,N> {
		let mut result = self.clone();
		result *= rhs;
		return result;
	}
}

impl<T: std::default::Default + std::marker::Copy + std::ops::Mul<Output = T> + std::ops::AddAssign<T>, const N: usize> ops::MulAssign<Self> for Mat<T,N> {
	fn mul_assign(&mut self, rhs: Self) {
		let result = (*self) * rhs;
		for i in 0..N {
			for j in 0..N {
				self.elements[i][j] = result.elements[i][j];
			}
		}
	}
}

impl<T: std::default::Default + std::marker::Copy + std::ops::Mul<Output = T> + std::ops::AddAssign<T>, const N: usize> ops::Mul<Self> for Mat<T,N> {

	type Output = Mat<T,N>;

	fn mul(self, rhs: Self) -> Mat<T,N> {

		let mut result = Mat::<T,N>::new();

		for i in 0..N {
			for j in 0..N {

				let mut sum: T = T::default();

				for k in 0..N {
					sum += self.elements[i][k] * rhs.elements[k][j];
				}

				result.elements[i][j] = sum;

			}
		}

		return result;

	}
}

impl<T: std::default::Default + std::marker::Copy, const N: usize> Mat<T,N> {

	pub fn new() -> Self {
		Mat{
			elements: [[T::default(); N]; N],
		}
	}

}

impl<const N: usize> Mat<f32,N> {

	pub fn scaling(
		factor: f32) -> Self
	{
		let mut result = Mat::<f32,N>::new();

		for i in 0..N {
			result.elements[i][i] = factor;
		}

		return result;
	}

}

impl Mat<f32,3> {

	pub fn rotation_z(
		theta: f32) -> Self
	{
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		return Mat{
			elements: [
				[cos_theta, 	sin_theta, 	0.0	],
				[-sin_theta, 	cos_theta, 	0.0	],
				[0.0, 			0.0, 		1.0	],
			]
		}
	}

	pub fn rotation_y(
		theta: f32) -> Self
	{
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		return Mat{
			elements: [
				[cos_theta, 	0.0, 	-sin_theta 	],
				[0.0, 			1.0, 	0.0 		],
				[sin_theta, 	0.0, 	cos_theta 	],
			]
		}
	}

	pub fn rotation_x(
		theta: f32) -> Self
	{
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		return Mat{
			elements: [
				[1.0, 	0.0, 	 	0.0 	 	],
				[0.0, 	cos_theta, 	sin_theta 	],
				[0.0, 	-sin_theta, cos_theta 	],
			]
		}
	}

}

impl Mat<f32,4> {

	pub fn rotation_z(
		theta: f32) -> Self
	{
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		return Mat{
			elements: [
				[cos_theta, 	sin_theta, 	0.0, 	0.0 ],
				[-sin_theta, 	cos_theta, 	0.0, 	0.0 ],
				[0.0, 			0.0, 		1.0, 	0.0	],
				[0.0, 			0.0, 		0.0, 	1.0	],
			]
		}
	}

	pub fn rotation_y(
		theta: f32) -> Self
	{
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		return Mat{
			elements: [
				[cos_theta, 	0.0, 	-sin_theta, 	0.0 ],
				[0.0, 			1.0, 	0.0, 			0.0 ],
				[sin_theta, 	0.0, 	cos_theta, 		0.0 ],
				[0.0, 			0.0, 	0.0, 			1.0 ],
			]
		}
	}

	pub fn rotation_x(
		theta: f32) -> Self
	{
		let sin_theta = theta.sin();
		let cos_theta = theta.cos();

		return Mat{
			elements: [
				[1.0, 	0.0, 	 	0.0, 		0.0  	],
				[0.0, 	cos_theta, 	sin_theta, 	0.0 	],
				[0.0, 	-sin_theta, cos_theta, 	0.0 	],
				[0.0, 	0.0, 		0.0, 		1.0 	],
			]
		}
	}

	pub fn translation(
		v: Vec3) -> Self
	{
		return Mat{
			elements: [
				[1.0, 	0.0,  0.0, 	0.0, ],
				[0.0, 	1.0,  0.0, 	0.0, ],
				[0.0, 	0.0,  1.0, 	0.0, ],
				[v.x, 	v.y,  v.z, 	1.0, ],
			]
		};
	}

	pub fn projection(
		width: f32,
		height: f32,
		near: f32,
		far: f32) -> Self
	{

		let (w, h, n, f) = (width, height, near, far);

		return Self {
			elements: [
				[2.0 * n / w as f32, 	0.0, 	 			 	0.0, 					0.0	],
				[0.0, 	 		 		2.0 * n / h as f32, 	0.0, 					0.0	],
				[0.0, 	 		 		0.0, 	 				f / (f - n), 	 		1.0	],
				[0.0, 	 		 		0.0, 	 				(-n * f) / (f - n), 	0.0	],
			]
		}
	}

}

pub type Mat3 = Mat<f32,3>;

impl ops::MulAssign<Mat3> for Vec3 {
	fn mul_assign(&mut self, rhs: Mat3) {

		let result = self.clone() * rhs;

		self.x = result.x;
		self.y = result.y;
		self.z = result.z;

	}
}

impl ops::Mul<Mat3> for Vec3 {
	type Output = Vec3;
	fn mul(self, rhs: Mat3) -> Self {
		Vec3 {
			x: (self.x * rhs.elements[0][0] + self.y * rhs.elements[1][0] + self.z * rhs.elements[2][0]),
			y: (self.x * rhs.elements[0][1] + self.y * rhs.elements[1][1] + self.z * rhs.elements[2][1]),
			z: (self.x * rhs.elements[0][2] + self.y * rhs.elements[1][2] + self.z * rhs.elements[2][2]),
		}
	}
}

pub type Mat4 = Mat<f32,4>;

impl ops::MulAssign<Mat4> for Vec4 {
	fn mul_assign(&mut self, rhs: Mat4) {

		let result = self.clone() * rhs;

		self.x = result.x;
		self.y = result.y;
		self.z = result.z;
		self.w = result.w;

	}
}

impl ops::Mul<Mat4> for Vec4 {
	type Output = Vec4;
	fn mul(self, rhs: Mat4) -> Self {
		Vec4 {
			x: (self.x * rhs.elements[0][0] + self.y * rhs.elements[1][0] + self.z * rhs.elements[2][0] + self.w * rhs.elements[3][0]),
			y: (self.x * rhs.elements[0][1] + self.y * rhs.elements[1][1] + self.z * rhs.elements[2][1] + self.w * rhs.elements[3][1]),
			z: (self.x * rhs.elements[0][2] + self.y * rhs.elements[1][2] + self.z * rhs.elements[2][2] + self.w * rhs.elements[3][2]),
			w: (self.x * rhs.elements[0][3] + self.y * rhs.elements[1][3] + self.z * rhs.elements[2][3] + self.w * rhs.elements[3][3]),
		}
	}
}