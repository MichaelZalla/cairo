use std::path::Path;

use crate::fs::read_lines;
use crate::vec::{vec2::Vec2, vec3::Vec3};

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
    let mut uv_coordinates: Vec<Vec2> = vec![];
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
                            // Comment
                            "#" => (),
                            // Geometric vertex, with (x, y, z, [w]) coordinates, w is optional and defaults to 1.0.
                            "v" => {
                                // `v  -0.512365 -40.559704 21.367237` (x y z)
                                // `v  -0.512365 -40.559704 21.367237 50 255 0` (x y z r g b)

                                let (x, y, z) = (
                                    line_components.next().unwrap().parse::<f32>().unwrap(),
                                    line_components.next().unwrap().parse::<f32>().unwrap(),
                                    line_components.next().unwrap().parse::<f32>().unwrap(),
                                );

                                vertices.push(Vec3 { x, y, z });
                            }
                            // Texture (UV) coordinate, as (u, [v, w]), between 0 and 1. v, w are optional and default to 0.
                            "vt" => {
                                // `vt 0.500 1 [0]` (u v w?)
                                let u = line_components.next().unwrap().parse::<f32>().unwrap();
                                let mut v = 0.0;
                                let mut w = 0.0;

                                let result = line_components.next();

                                match result {
                                    Some(value) => {
                                        v = value.parse::<f32>().unwrap();

                                        let result = line_components.next();

                                        match result {
                                            Some(value) => {
                                                w = value.parse::<f32>().unwrap();
                                            }
                                            None => (),
                                        }
                                    }
                                    None => (),
                                }

                                uv_coordinates.push(Vec2 { x: u, y: v, z: w })
                            }
                            // Vertex normal in (x,y,z) form; normal might not be a unit vector.
                            "vn" => {
                                // `vn  0.000005 -34.698460 -17.753405` (x y z)

                                let vertex_normal = Vec3 {
                                    x: line_components.next().unwrap().parse::<f32>().unwrap(),
                                    y: line_components.next().unwrap().parse::<f32>().unwrap(),
                                    z: line_components.next().unwrap().parse::<f32>().unwrap(),
                                };

                                vertex_normals.push(vertex_normal);
                            }
                            // Parameter space vertex
                            "vp" => (),
                            // Polygonal face
                            "f" => {
                                // Vertex indices only:             f v1 v2 v3 ....
                                // Vertex and UV indices:           f v1/vt1 v2/vt2 v3/vt3 ...
                                // Vertex, UV, and normal indices:  f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3 ...
                                // Vertex and normal indices only:  f v1//vn1 v2//vn2 v3//vn3 ...

                                // f 1 2 3
                                // f 3/1 4/2 5/3
                                // f 6/4/1 3/5/3 7/6/5
                                // f 7//1 8//2 9//3

                                // `f 1004//1004 1003//1003 1002//1002` ({x,y,z}{vert_index, texture_index, vert_normal_index})
                                // `f 1004//1004 1003//1003 1002//1002` ({x,y,z}{vert_index, texture_index, vert_normal_index})

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
                            // Line element
                            "l" => (),
                            // External material reference
                            "mtllib" => (),
                            // Material group
                            "usemtl" => (),
                            // Named object
                            "o" => (),
                            // Named object polygon group
                            "g" => (),
                            // Polygon smoothing group
                            "s" => (),
                            // Unrecognized prefix
                            other => {
                                println!("{}", other)
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Parsed mesh from OBJ file ({})", filepath);
    println!("  > Vertices: {}", vertices.len());
    println!("  > UV coordinates: {}", uv_coordinates.len());
    println!("  > Vertex normals: {}", vertex_normals.len());
    println!("  > Face vertex indices: {}", face_vertex_indices.len());
    println!(
        "  > Face vertex normal indices: {}",
        face_vertex_normal_indices.len()
    );

    return Mesh::new(
        vertices,
        vertex_normals,
        face_vertex_indices,
        face_vertex_normal_indices,
    );
}
