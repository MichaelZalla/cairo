use crate::vec::vec3::Vec3;

use super::{state_vector::StateVector, units::Newtons};

pub mod gravity;

pub type ContactPoint = Vec3;

pub type Force<T> = fn(state: &T, i: usize, current_time: f32) -> (Newtons, Option<ContactPoint>);

pub type BoxedForce<T> = Box<dyn Fn(&T, usize, f32) -> (Newtons, Option<ContactPoint>)>;

pub type PointForce = Force<StateVector>;
