use super::vec;

pub struct Mesh {
	pub v: Vec<vec::vec3::Vec3>,
	pub f: Vec<(usize, usize, usize)>,
}
