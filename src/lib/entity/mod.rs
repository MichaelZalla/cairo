use super::{
	vec::vec3::Vec3,
	mesh::Mesh
};

pub struct Entity {
	pub position: Vec3,
	pub rotation: Vec3,
	pub mesh: Mesh,
}

impl Entity {

	pub fn new(
		mesh: Mesh) -> Self
	{

		return Entity {
			mesh,
			position: Vec3::new(),
			rotation: Vec3::new(),
		}

	}

}
