use crate::vec::vec3::Vec3;

use super::{Mesh, Face};

pub fn make_box(
	width: f32,
	height: f32,
	depth: f32) -> Mesh {

	let front_top_left = Vec3 {
		x: -width / 2.0,
		y: height / 2.0,
		z: depth / 2.0,
	};

	let front_top_right = Vec3 {
		x: width / 2.0,
		y: height / 2.0,
		z: depth / 2.0,
	};

	let front_bottom_left = Vec3 {
		x: -width / 2.0,
		y: -height / 2.0,
		z: depth / 2.0,
	};

	let front_bottom_right = Vec3 {
		x: width / 2.0,
		y: -height / 2.0,
		z: depth / 2.0,
	};

	let mut back_top_left = front_top_left.clone();

	back_top_left.z -= depth;

	let mut back_top_right = front_top_right.clone();

	back_top_right.z -= depth;

	let mut back_bottom_left = front_bottom_left.clone();

	back_bottom_left.z -= depth;

	let mut back_bottom_right = front_bottom_right.clone();

	back_bottom_right.z -= depth;

	let vertices: Vec<Vec3> = vec![
		front_top_left, 	// 0
		front_top_right, 	// 1
		front_bottom_left, 	// 2
		front_bottom_right, // 3
		back_top_left, 		// 4
		back_top_right, 	// 5
		back_bottom_left, 	// 6
		back_bottom_right, 	// 7
	];

	let mut faces: Vec<Face> = vec![];

	// Front face

	faces.push((0, 2, 1));
	faces.push((2, 3, 1));

	// Back face

	faces.push((4, 5, 6));
	faces.push((5, 7, 6));

	// Top face

	faces.push((4, 0, 5));
	faces.push((0, 1, 5));

	// Bottom face

	faces.push((6, 7, 2));
	faces.push((7, 3, 2));

	// Left face

	faces.push((4, 6, 0));
	faces.push((6, 2, 0));

	// Right face

	faces.push((1, 3, 7));
	faces.push((7, 5, 1));

	// Generates dummy normals to prevent vertex-duplication per face;

	let mut vertex_normals: Vec<Vec3> = vec![];

	for _ in vertices.as_slice() {
		vertex_normals.push(Vec3 { x: 0.0, y: 1.0, z: 0.0 })
	}

	return Mesh::new(
		vertices,
		faces,
		vertex_normals,
		vec![]
	);

}
