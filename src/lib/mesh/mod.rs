use std::fs::File;
use std::io::{self, BufRead};

use std::path::Path;

use super::vec::vec3::Vec3;

pub type Face = (usize, usize, usize);

pub struct Mesh {
	pub v: Vec<Vec3>,
	pub f: Vec<Face>,
	pub vn: Vec<Vec3>,
	pub tn: Vec<(usize, usize, usize)>,
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

	let mesh: Mesh = Mesh{
		v: vertices,
		f: faces,
		vn: vertex_normals,
		tn: face_normals,
	};

	println!(
		"Compiled mesh with {} vertices, {} faces, {} vertex normals, and {} face normals.",
		mesh.v.len(),
		mesh.f.len(),
		mesh.vn.len(),
		mesh.tn.len(),
	);

	return mesh;

}

fn read_lines<P>(filepath: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {

	let file = File::open(filepath)?;

	Ok(io::BufReader::new(file).lines())

}
