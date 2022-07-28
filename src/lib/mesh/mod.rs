use std::fs::File;
use std::io::{self, BufRead};

use std::path::Path;

use crate::vertices::default_vertex_in::DefaultVertexIn;

use super::vec::vec3::Vec3;

pub type Face = (usize, usize, usize);

pub struct Mesh {
	pub vertices: Vec<DefaultVertexIn>,
	pub face_indices: Vec<Face>,
}

pub fn get_mesh_from_obj(
	filepath: String) -> Mesh
{

	let path = Path::new(&filepath);

	let display = path.display();

	let lines = match read_lines(&path) {
		Err(why) => panic!("Failed to open file {}: {}", display, why),
		Ok(lines) => lines,
	};

	let mut vertices: Vec<Vec3> = vec![];
	let mut faces: Vec<Face> = vec![];
	let mut vertex_normals: Vec<Vec3> = vec![];
	let mut face_normals: Vec<(usize, usize, usize)> = vec![];

	for (_, line) in lines.enumerate() {
		match line {
			Err(why) => println!("Error reading next line: {}", why),
			Ok(line) => {

				let mut line_components = line.split_whitespace();

				match line_components.next() {
					None => (),
					Some(first) => {
						match first {
							"v" => {

								// `v  -0.512365 -40.559704 21.367237`

								let (x, y, z) = (
									line_components.next().unwrap().parse::<f32>().unwrap(),
									line_components.next().unwrap().parse::<f32>().unwrap(),
									line_components.next().unwrap().parse::<f32>().unwrap(),
								);

								vertices.push(Vec3{ x, y, z });

							},
							"f" => {

								// `f 1004//1004 1003//1003 1002//1002`

								let mut x = line_components.next().unwrap().split("/");
								let mut y = line_components.next().unwrap().split("/");
								let mut z = line_components.next().unwrap().split("/");

								faces.push((
									x.next().unwrap().parse::<usize>().unwrap() - 1,
									y.next().unwrap().parse::<usize>().unwrap() - 1,
									z.next().unwrap().parse::<usize>().unwrap() - 1,
								));

								let result = x.next();

								match result {
									Some(_) => {

										y.next();
										z.next();

										face_normals.push((
											x.next().unwrap().parse::<usize>().unwrap() - 1,
											y.next().unwrap().parse::<usize>().unwrap() - 1,
											z.next().unwrap().parse::<usize>().unwrap() - 1,
										));

									},
									None => (),
								}

							},
							"vn" => {

								// `vn  0.000005 -34.698460 -17.753405`

								let vertex_normal = Vec3{
									x: line_components.next().unwrap().parse::<f32>().unwrap(),
									y: line_components.next().unwrap().parse::<f32>().unwrap(),
									z: line_components.next().unwrap().parse::<f32>().unwrap(),
								};

								vertex_normals.push(vertex_normal);

							},
							"#" => (),
							// "mtllib" => println!("mtllib"),
							// "ysentk" => println!("ysentk"),
							// "o" => println!("o"),
							// "g" => println!("g"),
							// "s" => println!("s"),
							other => println!("{}", other),
						}
					}
				}
			},
		}
	}

	let mesh_v_len = vertices.len();
	let mesh_vn_len = vertex_normals.len();
	let mesh_tn_len = face_normals.len();

	let mut mesh: Mesh = Mesh{
		vertices: vec![],
		face_indices: vec![],
	};

	let white = Vec3{
		x: 1.0,
		y: 1.0,
		z: 1.0,
	};

	if mesh_tn_len == faces.len() {

		// Case 1. 3 vertex normals are defined per face;

		println!("Case 1!");

		for (face_index, face) in faces.iter().enumerate() {

			let normal_indices = face_normals[face_index];

			mesh.vertices.push(DefaultVertexIn {
				p: vertices[face.0].clone(),
				n: vertex_normals[normal_indices.0].clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			});

			mesh.vertices.push(DefaultVertexIn {
				p: vertices[face.1].clone(),
				n: vertex_normals[normal_indices.1].clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			});

			mesh.vertices.push(DefaultVertexIn {
				p: vertices[face.2].clone(),
				n: vertex_normals[normal_indices.2].clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			});

			mesh.face_indices.push((
				face_index * 3,
				face_index * 3 + 1,
				face_index * 3 + 2,
			))

		}

	} else if mesh_vn_len != mesh_v_len {

		// Case 2. No normal data was provided; we'll generate a normal for each
		// face, creating 3 unique Vertex instances for that face;

		println!("Case 2!");

		for (face_index, face) in faces.iter().enumerate() {

			let computed_normal = (vertices[face.1] - vertices[face.0])
				.cross(vertices[face.2] - vertices[face.0])
				.as_normal();

			mesh.vertices.push(DefaultVertexIn {
				p: vertices[face.0].clone(),
				n: computed_normal.clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			});

			mesh.vertices.push(DefaultVertexIn {
				p: vertices[face.1].clone(),
				n: computed_normal.clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			});

			mesh.vertices.push(DefaultVertexIn {
				p: vertices[face.2].clone(),
				n: computed_normal.clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			});

			mesh.face_indices.push((
				face_index * 3,
				face_index * 3 + 1,
				face_index * 3 + 2,
			))

		}

	}

	if mesh_vn_len == mesh_v_len {

		println!("Case 3!");

		// Case 3. One normal is defined per-vertex; no need for duplicate Vertexs;

		for (vertex_index, vertex) in vertices.iter().enumerate() {

			mesh.vertices.push(DefaultVertexIn {
				p: vertex.clone(),
				n: vertex_normals[vertex_index].clone(),
				c: white.clone(),
				world_pos: Vec3::new(),
			})

		}

		mesh.face_indices = faces;

	}

	println!(
		"Compiled mesh with {} vertices, {} faces, {} vertex normals, and {} face normals.",
		mesh.vertices.len(),
		mesh.face_indices.len(),
		mesh_vn_len,
		mesh_tn_len,
	);

	return mesh;

}

fn read_lines<P>(filepath: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {

	let file = File::open(filepath)?;

	Ok(io::BufReader::new(file).lines())

}
