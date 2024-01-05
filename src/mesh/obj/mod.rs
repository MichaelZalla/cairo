use std::path::Path;

use crate::fs::read_lines;

use crate::material;
use crate::mesh::{Face, MaterialSource};
use crate::vec::{vec2::Vec2, vec3::Vec3};

use super::Mesh;

pub fn load_obj(filepath: &str) -> Vec<Mesh> {
    let path = Path::new(&filepath);
    let path_display = path.display();
    let path_parent = path.parent().unwrap();

    let lines = match read_lines(&path) {
        Err(why) => panic!("Failed to open file {}: {}", path_display, why),
        Ok(lines) => lines,
    };

    let mut object_name: Option<String> = None;
    let mut group_name: Option<String> = None;
    let mut material_source: Option<MaterialSource> = None;
    let mut material_name: Option<String> = None;

    let mut vertices: Vec<Vec3> = vec![];
    let mut normals: Vec<Vec3> = vec![];
    let mut uvs: Vec<Vec2> = vec![];
    let mut faces: Vec<Face> = vec![];

    // Counters

    let mut vertex_counter: usize = 0;
    let mut uv_counter: usize = 0;
    let mut normal_counter: usize = 0;
    let mut object_counter: usize = 0;
    let mut group_counter: usize = 0;

    for (_, line) in lines.enumerate() {
        match line {
            Err(why) => println!("Error reading next line: {}", why),
            Ok(line) => {
                let mut line_tokens = line.split_whitespace();

                match line_tokens.next() {
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
                                    line_tokens.next().unwrap().parse::<f32>().unwrap(),
                                    line_tokens.next().unwrap().parse::<f32>().unwrap(),
                                    line_tokens.next().unwrap().parse::<f32>().unwrap(),
                                );

                                vertices.push(Vec3 { x, y, z });

                                vertex_counter += 1;
                            }
                            // Texture (UV) coordinate, as (u, [v, w]), between 0 and 1. v, w are optional and default to 0.
                            "vt" => {
                                // `vt 0.500 1 [0]` (u v w?)
                                let u = line_tokens.next().unwrap().parse::<f32>().unwrap();
                                let mut v = 0.0;
                                let mut w = 0.0;

                                let result = line_tokens.next();

                                match result {
                                    Some(value) => {
                                        v = value.parse::<f32>().unwrap();

                                        let result = line_tokens.next();

                                        match result {
                                            Some(value) => {
                                                w = value.parse::<f32>().unwrap();
                                            }
                                            None => (),
                                        }
                                    }
                                    None => (),
                                }

                                uvs.push(Vec2 { x: u, y: v, z: w });

                                uv_counter += 1;
                            }
                            // Vertex normal in (x,y,z) form; normal might not be a unit vector.
                            "vn" => {
                                // `vn  0.000005 -34.698460 -17.753405` (x y z)

                                let vertex_normal = Vec3 {
                                    x: line_tokens.next().unwrap().parse::<f32>().unwrap(),
                                    y: line_tokens.next().unwrap().parse::<f32>().unwrap(),
                                    z: line_tokens.next().unwrap().parse::<f32>().unwrap(),
                                };

                                normals.push(vertex_normal);

                                normal_counter += 1;
                            }
                            // Parameter space vertex
                            "vp" => (),
                            // Polygonal face
                            "f" => {
                                // Vertex indices only:             f v1 v2 v3 ....
                                // Vertex and UV indices:           f v1/uv1 v2/uv2 v3/uv3 ...
                                // Vertex, UV, and normal indices:  f v1/uv1/n1 v2/uv2/n2 v3/uv3/n3 ...
                                // Vertex and normal indices only:  f v1//n1 v2//n2 v3//n3 ...

                                // f 1 2 3
                                // f 3/1 4/2 5/3
                                // f 6/4/1 3/5/3 7/6/5
                                // f 7//1 8//2 9//3

                                // `f 1004//1004 1003//1003 1002//1002` ({x,y,z}{vert_index, texture_index, vert_normal_index})
                                // `f 1004//1004 1003//1003 1002//1002` ({x,y,z}{vert_index, texture_index, vert_normal_index})

                                let mut face: Face = Default::default();

                                let mut v1_iter = line_tokens.next().unwrap().split("/");
                                let mut v2_iter = line_tokens.next().unwrap().split("/");
                                let mut v3_iter = line_tokens.next().unwrap().split("/");

                                face.vertices = (
                                    v1_iter.next().unwrap().parse::<usize>().unwrap() - 1,
                                    v2_iter.next().unwrap().parse::<usize>().unwrap() - 1,
                                    v3_iter.next().unwrap().parse::<usize>().unwrap() - 1,
                                );

                                let v1_uv_index = v1_iter.next();
                                let v2_uv_index = v2_iter.next();
                                let v3_uv_index = v3_iter.next();

                                match v1_uv_index {
                                    Some(index) => {
                                        if index != "" {
                                            let v1_uv =
                                                v1_uv_index.unwrap().parse::<usize>().unwrap() - 1;
                                            let v2_uv =
                                                v2_uv_index.unwrap().parse::<usize>().unwrap() - 1;
                                            let v3_uv =
                                                v3_uv_index.unwrap().parse::<usize>().unwrap() - 1;

                                            face.uvs = Some((v1_uv, v2_uv, v3_uv));
                                        }
                                    }
                                    None => (),
                                }

                                let v1_normal_index = v1_iter.next();

                                match v1_normal_index {
                                    Some(_) => {
                                        let v2_normal_index = v2_iter.next();
                                        let v3_normal_index = v3_iter.next();

                                        let v1_n =
                                            v1_normal_index.unwrap().parse::<usize>().unwrap() - 1;
                                        let v2_n =
                                            v2_normal_index.unwrap().parse::<usize>().unwrap() - 1;
                                        let v3_n =
                                            v3_normal_index.unwrap().parse::<usize>().unwrap() - 1;

                                        face.normals = Some((v1_n, v2_n, v3_n));
                                    }
                                    None => (),
                                }

                                faces.push(face);
                            }
                            // Line element
                            "l" => (),
                            // External material reference
                            "mtllib" => {
                                let mtl_filepath = line_tokens.next().unwrap();

                                let mtl_path_relative = path_parent
                                    .join(mtl_filepath)
                                    .into_os_string()
                                    .into_string()
                                    .unwrap();
                                let mtl_path_relative_str = mtl_path_relative.as_str();

                                material_source = Some(MaterialSource {
                                    filepath: mtl_path_relative_str.to_string(),
                                });
                            }
                            // Material group
                            "usemtl" => {
                                let name = line_tokens.next().unwrap();

                                material_name = Some(name.to_string());
                            }
                            // Named object
                            "o" => {
                                let name = line_tokens.next().unwrap();

                                object_counter += 1;

                                object_name = Some(name.to_string());
                            }
                            // Named object polygon group
                            "g" => {
                                let name = line_tokens.next().unwrap();

                                group_counter += 1;

                                group_name = Some(name.to_string());
                            }
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

    let mut meshes = vec![Mesh::new(vertices, uvs, normals, faces)];

    meshes.last_mut().unwrap().object_source = path_display.to_string();

    match object_name {
        Some(name) => {
            meshes.last_mut().unwrap().object_name = name;
        }
        None => (),
    }

    match group_name {
        Some(name) => {
            meshes.last_mut().unwrap().group_name = name;
        }
        None => (),
    }

    meshes.last_mut().unwrap().material_source = material_source;

    match material_name {
        Some(name) => {
            meshes.last_mut().unwrap().material_name = name;
        }
        None => (),
    }

    let count: usize = meshes.len();

    println!(
        "Parsed {} mesh{} from \"{}\":",
        count,
        if count > 1 { "es" } else { "" },
        path_display
    );

    println!(
        "Counted {} objects, {} groups, {} vertices, {} UVs, {} normals.",
        object_counter, group_counter, vertex_counter, uv_counter, normal_counter
    );

    println!();

    for mesh in meshes.as_mut_slice() {
        // Load any materials from associated MTL files

        match &mesh.material_source {
            Some(src) => {
                // Parse the set of materials inside the MTL source
                let materials = material::mtl::load_mtl(&src.filepath);

                // Find the material referenced by this mesh (via usemtl)
                for i in 0..materials.len() {
                    let mat = &materials[i];
                    if mat.name == mesh.material_name {
                        mesh.material = Some(mat.to_owned());
                    }
                }
            }
            None => (),
        }

        println!("{}", mesh);
    }

    return meshes;
}
