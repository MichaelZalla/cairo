use std::path::Path;

use crate::{mesh::read_lines, vec::vec3::Vec3};

use super::Mesh;

pub fn get_mesh_from_obj(filepath: String) -> Mesh {
    let path = Path::new(&filepath);

    let display = path.display();

    let lines = match read_lines(&path) {
        Err(why) => panic!("Failed to open file {}: {}", display, why),
        Ok(lines) => lines,
    };

    let mut vertices: Vec<Vec3> = vec![];
    let mut vertex_normals: Vec<Vec3> = vec![];
    let mut face_vertex_indices: Vec<(usize, usize, usize)> = vec![];
    let mut face_vertex_normal_indices: Vec<(usize, usize, usize)> = vec![];

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

                                vertices.push(Vec3 { x, y, z });
                            }
                            "f" => {
                                // `f 1004//1004 1003//1003 1002//1002`

                                let mut x = line_components.next().unwrap().split("/");
                                let mut y = line_components.next().unwrap().split("/");
                                let mut z = line_components.next().unwrap().split("/");

                                face_vertex_indices.push((
                                    x.next().unwrap().parse::<usize>().unwrap() - 1,
                                    y.next().unwrap().parse::<usize>().unwrap() - 1,
                                    z.next().unwrap().parse::<usize>().unwrap() - 1,
                                ));

                                let result = x.next();

                                match result {
                                    Some(_) => {
                                        y.next();
                                        z.next();

                                        face_vertex_normal_indices.push((
                                            x.next().unwrap().parse::<usize>().unwrap() - 1,
                                            y.next().unwrap().parse::<usize>().unwrap() - 1,
                                            z.next().unwrap().parse::<usize>().unwrap() - 1,
                                        ));
                                    }
                                    None => (),
                                }
                            }
                            "vn" => {
                                // `vn  0.000005 -34.698460 -17.753405`

                                let vertex_normal = Vec3 {
                                    x: line_components.next().unwrap().parse::<f32>().unwrap(),
                                    y: line_components.next().unwrap().parse::<f32>().unwrap(),
                                    z: line_components.next().unwrap().parse::<f32>().unwrap(),
                                };

                                vertex_normals.push(vertex_normal);
                            }
                            "#" => (),
                            // "mtllib" => println!("mtllib"),
                            // "ysentk" => println!("ysentk"),
                            // "o" => println!("o"),
                            // "g" => println!("g"),
                            // "s" => println!("s"),
                            _ => {
                                // println!("{}", other)
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{}", filepath,);

    println!(
        "  Parsed mesh with {} vertices, {} vertex normals, {} faces, and {} face normals.",
        vertices.len(),
        vertex_normals.len(),
        face_vertex_indices.len(),
        face_vertex_normal_indices.len(),
    );

    return Mesh::new(
        vertices,
        vertex_normals,
        face_vertex_indices,
        face_vertex_normal_indices,
    );
}
