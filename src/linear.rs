#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}

pub struct Vec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

pub struct Mesh {
	pub v: Vec<Vec3>,
	pub f: Vec<(usize, usize, usize)>,
}
