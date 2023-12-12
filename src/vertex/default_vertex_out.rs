use std::{
	fmt::{Display, Formatter, Result},
	ops::{Add, Sub, Mul, MulAssign, Div}
};

use crate::vec::{vec3::Vec3, vec4::Vec4};

#[derive(Copy, Clone, Default)]
pub struct DefaultVertexOut {
	pub p: Vec4,
	pub n: Vec4,
	pub c: Vec3,
	pub world_pos: Vec3,
}

impl DefaultVertexOut {

	pub fn new() -> Self {
		Default::default()
	}

	pub fn interpolate(
		start: Self,
		end: Self,
		alpha: f32) -> Self
	{
		return start + (end - start) * alpha;
	}

}

impl Add<DefaultVertexOut> for DefaultVertexOut {
	type Output = DefaultVertexOut;
	fn add(self, rhs: Self) -> DefaultVertexOut {
		DefaultVertexOut {
			p: self.p + rhs.p,
			n: self.n + rhs.n,
			c: self.c + rhs.c,
			world_pos: self.world_pos + rhs.world_pos,
		}
	}
}

impl Sub<DefaultVertexOut> for DefaultVertexOut {
	type Output = DefaultVertexOut;
	fn sub(self, rhs: Self) -> DefaultVertexOut {
		DefaultVertexOut {
			p: self.p - rhs.p,
			n: self.n - rhs.n,
			c: self.c - rhs.c,
			world_pos: self.world_pos - rhs.world_pos,
		}
	}
}

impl Mul<f32> for DefaultVertexOut {
	type Output = DefaultVertexOut;
	fn mul(self, scalar: f32) -> DefaultVertexOut {
		DefaultVertexOut {
			p: self.p * scalar,
			n: self.n * scalar,
			c: self.c * scalar,
			world_pos: self.world_pos * scalar,
		}
	}
}

impl MulAssign<f32> for DefaultVertexOut {
	fn mul_assign(&mut self, scalar: f32) {
		self.p *= scalar;
		self.n *= scalar;
		self.c *= scalar;
		self.world_pos *= scalar;
	}
}

impl Div<f32> for DefaultVertexOut {
	type Output = DefaultVertexOut;
	fn div(self, scalar: f32) -> DefaultVertexOut {
		DefaultVertexOut {
			p: self.p / scalar,
			n: self.n / scalar,
			c: self.c / scalar,
			world_pos: self.world_pos / scalar,
		}
	}
}

impl Display for DefaultVertexOut {
	fn fmt(&self, v: &mut Formatter<'_>) -> Result {
		write!(v, "{}", self.p)
    }
}
