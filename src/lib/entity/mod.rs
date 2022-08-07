use super::{
	vec::vec3::Vec3,
	mesh::{
		Mesh,
		primitive::make_box,
	}
};

#[derive(Default, Clone)]
pub struct Entity {
	pub position: Vec3,
	pub rotation: Vec3,
	pub mesh: Mesh,
	pub collider: Mesh,
}

impl Entity {

	pub fn new(
		mesh: Mesh) -> Self
	{

		let collider = Entity::make_collider(&mesh);

		return Entity {
			position: Vec3::new(),
			rotation: Vec3::new(),
			mesh,
			collider,
		};

	}

	fn make_collider(
		mesh: &Mesh) -> Mesh
	{

		let mut x_min: f32 = f32::MAX;
		let mut x_max: f32 = f32::MIN;

		let mut y_min: f32 = f32::MAX;
		let mut y_max: f32 = f32::MIN;

		let mut z_min: f32 = f32::MAX;
		let mut z_max: f32 = f32::MIN;

		for v in mesh.vertices.as_slice() {

			if v.p.x < x_min {
				x_min = v.p.x;
			} else if v.p.x > x_max {
				x_max = v.p.x;
			}

			if v.p.y < y_min {
				y_min = v.p.y;
			} else if v.p.y > y_max {
				y_max = v.p.y;
			}

			if v.p.z < z_min {
				z_min = v.p.z;
			} else if v.p.z > z_max {
				z_max = v.p.z;
			}

		}

		let width = x_max - x_min;
		let height = y_max - y_min;
		let depth = z_max - z_min;

		let mut collider = make_box(
			width,
			height,
			depth,
		);

		let collider_offset = Vec3 {
			x: x_min + width / 2.0,
			y: y_min + height / 2.0,
			z: z_min + depth / 2.0,
		};

		let yellow = Vec3 {
			x: 1.0,
			y: 1.0,
			z: 0.0,
		};

		for v in collider.vertices.as_mut_slice() {
			v.p += collider_offset;
			v.c = yellow.clone();
		}

		return collider;

	}

}
