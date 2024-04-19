use std::collections::HashSet;
use std::io::Error;
use std::path::Path;

use crate::fs::read_lines;

use crate::material;
use crate::material::cache::MaterialCache;
use crate::mesh::{
    geometry::{Face, Geometry},
    Mesh,
};
use crate::{
    resource::arena::Arena,
    texture::map::TextureMap,
    vec::{vec2::Vec2, vec3::Vec3},
};

pub fn load_obj(
    filepath: &str,
    texture_arena: &mut Arena<TextureMap>,
) -> (Vec<Mesh>, Option<MaterialCache>) {
    let path = Path::new(&filepath);
    let path_display = path.display();
    let path_parent = path.parent().unwrap();

    let lines = match read_lines(&path) {
        Err(why) => panic!("Failed to open file {}: {}", path_display, why),
        Ok(lines) => lines,
    };

    let end_signal: std::vec::IntoIter<Result<String, Error>> =
        vec![(Ok("o __default__".to_string()))].into_iter();

    let chained = lines.chain(end_signal);

    let mut objects: Vec<Mesh> = vec![];

    let mut material_source: Option<String> = None;

    // Global state

    let mut normals: Vec<Vec3> = vec![];

    // Current object state

    let mut group_name: Option<String> = None;
    let mut object_name: Option<String> = None;
    let mut material_name: Option<String> = None;

    let mut object_vertices: Vec<Vec3> = vec![];
    let mut object_uvs: Vec<Vec2> = vec![];
    let mut object_faces: Vec<Face> = vec![];

    // Counters

    let mut vertex_counter: usize = 0;
    let mut uv_counter: usize = 0;
    let mut normal_counter: usize = 0;
    let mut face_counter: usize = 0;
    let mut object_counter: usize = 0;
    let mut group_counter: usize = 0;

    let mut vertex_index_offset_for_current_object: usize = 0;
    let mut uv_index_offset_for_current_object: usize = 0;

    for (_, line) in chained.enumerate() {
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

                                object_vertices.push(Vec3 { x, y, z });

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

                                object_uvs.push(Vec2 { x: u, y: v, z: w });

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
                                    v1_iter.next().unwrap().parse::<usize>().unwrap()
                                        - 1
                                        - vertex_index_offset_for_current_object,
                                    v2_iter.next().unwrap().parse::<usize>().unwrap()
                                        - 1
                                        - vertex_index_offset_for_current_object,
                                    v3_iter.next().unwrap().parse::<usize>().unwrap()
                                        - 1
                                        - vertex_index_offset_for_current_object,
                                );

                                let v1_uv_index = v1_iter.next();
                                let v2_uv_index = v2_iter.next();
                                let v3_uv_index = v3_iter.next();

                                match v1_uv_index {
                                    Some(index) => {
                                        if index != "" {
                                            let v1_uv =
                                                v1_uv_index.unwrap().parse::<usize>().unwrap()
                                                    - 1
                                                    - uv_index_offset_for_current_object;
                                            let v2_uv =
                                                v2_uv_index.unwrap().parse::<usize>().unwrap()
                                                    - 1
                                                    - uv_index_offset_for_current_object;
                                            let v3_uv =
                                                v3_uv_index.unwrap().parse::<usize>().unwrap()
                                                    - 1
                                                    - uv_index_offset_for_current_object;

                                            if v1_uv > object_uvs.len() - 1
                                                || v2_uv > object_uvs.len() - 1
                                                || v3_uv > object_uvs.len() - 1
                                            {
                                                panic!("Invalid UV indices ({},{},{}) for UV group with size {}.", v1_uv, v2_uv, v3_uv, object_uvs.len())
                                            }

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

                                        let v1_n_raw =
                                            v1_normal_index.unwrap().parse::<usize>().unwrap();
                                        let v2_n_raw =
                                            v2_normal_index.unwrap().parse::<usize>().unwrap();
                                        let v3_n_raw =
                                            v3_normal_index.unwrap().parse::<usize>().unwrap();

                                        let v1_n = v1_n_raw - 1;
                                        let v2_n = v2_n_raw - 1;
                                        let v3_n = v3_n_raw - 1;

                                        face.normals = Some((v1_n, v2_n, v3_n));
                                    }
                                    None => (),
                                }

                                object_faces.push(face);
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

                                material_source = Some(mtl_path_relative_str.to_string());
                            }
                            // Material group
                            "usemtl" => {
                                let name = line_tokens.next().unwrap();

                                material_name = Some(name.to_string());
                            }
                            // Named object
                            "o" => {
                                // Assemble any previous mesh before continuing to this object

                                vertex_index_offset_for_current_object = vertex_counter;
                                uv_index_offset_for_current_object = uv_counter;

                                // Coallesce all of this object's face normal indices into a set.

                                let mut object_normal_indices_set = HashSet::<usize>::new();

                                for face in &object_faces {
                                    match face.normals {
                                        Some(normals) => {
                                            object_normal_indices_set.insert(normals.0);
                                            object_normal_indices_set.insert(normals.1);
                                            object_normal_indices_set.insert(normals.2);
                                        }
                                        None => (),
                                    }
                                }

                                // Collect the set of unique normal indices into a Vec<usize>.

                                let object_normal_indices_vec: Vec<usize> =
                                    object_normal_indices_set.into_iter().collect();

                                // Copy the set of all referenced normals into a Vec<Vec3>.

                                let mut object_normals: Vec<Vec3> = vec![];

                                for index in &object_normal_indices_vec {
                                    object_normals.push(normals[*index].clone());
                                }

                                // Re-map each of the object's face normal
                                // indices to its index in object_normals[].

                                for face in &mut object_faces {
                                    match &mut face.normals {
                                        Some(normals) => {
                                            let global_index_0 = normals.0;
                                            let global_index_1 = normals.1;
                                            let global_index_2 = normals.2;

                                            match &object_normal_indices_vec
                                                .iter()
                                                .position(|&item| item == global_index_0)
                                            {
                                                Some(index) => {
                                                    normals.0 = *index;
                                                }
                                                None => {
                                                    panic!("Failed to find index {} in object normal indices list!", {global_index_0});
                                                }
                                            }

                                            match &object_normal_indices_vec
                                                .iter()
                                                .position(|&item| item == global_index_1)
                                            {
                                                Some(index) => {
                                                    normals.1 = *index;
                                                }
                                                None => {
                                                    panic!("Failed to find index {} in object normal indices list!", {global_index_1});
                                                }
                                            }

                                            match &object_normal_indices_vec
                                                .iter()
                                                .position(|&item| item == global_index_2)
                                            {
                                                Some(index) => {
                                                    normals.2 = *index;
                                                }
                                                None => {
                                                    panic!("Failed to find index {} in object normal indices list!", {global_index_2});
                                                }
                                            }
                                        }
                                        None => (),
                                    }
                                }

                                if vertex_index_offset_for_current_object > 0 {
                                    let mut accumulated_geometry = Geometry::new(
                                        object_vertices,
                                        object_uvs,
                                        object_normals,
                                        object_faces.clone(),
                                    );

                                    accumulated_geometry.object_source =
                                        Some(path_display.to_string());

                                    match object_name.to_owned() {
                                        Some(name) => {
                                            accumulated_geometry.object_name = Some(name);
                                        }
                                        None => (),
                                    }

                                    match group_name.to_owned() {
                                        Some(name) => {
                                            accumulated_geometry.group_name = Some(name);
                                        }
                                        None => (),
                                    }

                                    accumulated_geometry.material_source = material_source.clone();

                                    match material_name.to_owned() {
                                        Some(name) => {
                                            accumulated_geometry.material_name = Some(name);
                                        }
                                        None => (),
                                    }

                                    println!(
                                        "Parsed object {}.",
                                        accumulated_geometry
                                            .object_name
                                            .as_ref()
                                            .unwrap_or(&"Unnamed".to_string())
                                    );

                                    objects.push(Mesh::new(accumulated_geometry));

                                    object_counter += 1;
                                }

                                // Reset our state and accumulators

                                object_vertices = vec![];
                                object_uvs = vec![];

                                group_name = None;
                                material_name = None;

                                let name = line_tokens.next().unwrap();

                                object_name = Some(name.to_string());
                            }
                            // Named object polygon group
                            "g" => {
                                face_counter += object_faces.len();

                                object_faces = vec![];

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

    let object_count: usize = objects.len();

    println!(
        "Parsed {} object{} from \"{}\":",
        object_count,
        if object_count > 1 { "s" } else { "" },
        path_display
    );

    println!(
        "Counted {} objects, {} groups, {} vertices, {} faces, {} UVs, {} normals.",
        object_counter, group_counter, vertex_counter, face_counter, uv_counter, normal_counter
    );

    println!();

    for mesh in objects.as_mut_slice() {
        // Print a summary of this Mesh.

        println!("{:?}", mesh.geometry.object_name);
    }

    // Parse the set of materials inside this OBJ file's MTL file

    match &material_source {
        Some(src) => {
            let material_cache = material::mtl::load_mtl(&src, texture_arena);

            (objects, Some(material_cache))
        }
        None => (objects, None),
    }
}
