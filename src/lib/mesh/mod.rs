use std::rc::{Rc};
use std::cell::{RefCell};

use super::vec;

pub struct Mesh {
	pub v: Vec<vec::vec3::Vec3>,
	pub f: Vec<(usize, usize, usize)>,
	pub c: Rc<RefCell<Vec<f32>>>,
}
