use std::{fmt::{Display, Formatter, Result}, ops::{Add, Sub, Mul, Div}};

use crate::lib::vec::vec3::Vec3;

#[derive(Copy, Clone, Default)]
pub struct DefaultVertex {
	pub p: Vec3,
	pub n: Vec3,
}

impl DefaultVertex {

	pub fn new() -> Self {
		Default::default()
	}

	pub fn interpolate(
		start: DefaultVertex,
		end: DefaultVertex,
		alpha: f32) -> DefaultVertex
	{
		return start + (end - start) * alpha;
	}

}

impl Add<DefaultVertex> for DefaultVertex {
	type Output = DefaultVertex;
	fn add(self, rhs: DefaultVertex) -> DefaultVertex {
		DefaultVertex {
			p: self.p + rhs.p,
			n: self.n + rhs.n,
		}
	}
}

impl Sub<DefaultVertex> for DefaultVertex {
	type Output = DefaultVertex;
	fn sub(self, rhs: DefaultVertex) -> DefaultVertex {
		DefaultVertex {
			p: self.p - rhs.p,
			n: self.n - rhs.n,
		}
	}
}

impl Mul<f32> for DefaultVertex {
	type Output = DefaultVertex;
	fn mul(self, scalar: f32) -> DefaultVertex {
		DefaultVertex {
			p: self.p * scalar,
			n: self.n * scalar,
		}
	}
}

impl Div<f32> for DefaultVertex {
	type Output = DefaultVertex;
	fn div(self, scalar: f32) -> DefaultVertex {
		DefaultVertex {
			p: self.p / scalar,
			n: self.n / scalar,
		}
	}
}

impl Display for DefaultVertex {
	fn fmt(&self, v: &mut Formatter<'_>) -> Result {
		write!(v, "{}", self.p)
    }
}
