use std::{fmt, io::Error, mem, path::Path, rc::Rc};

use crate::{
    fs::read_lines,
    material::{mtl::load_mtl, Material},
    mesh::{
        geometry::Geometry,
        obj::parse::{
            parse_face, parse_mtllib, parse_vertex, parse_vertex_normal, parse_vertex_uv,
        },
        Mesh, PartialFace,
    },
    resource::arena::Arena,
    texture::map::TextureMap,
    vec::{vec2::Vec2, vec3::Vec3},
};

pub struct LoadObjResult(pub Rc<Geometry>, pub Vec<Mesh>);

#[derive(Default, Debug)]
struct LoadObjStats {
    object: usize,
    group: usize,
    material_group: usize,
    vertex: usize,
    uv: usize,
    normal: usize,
    face: usize,
}

impl fmt::Display for LoadObjStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counted {} objects, {} groups, {} material groups, {} vertices, {} UVs, {} normals, and {} faces.", self.object, self.group, self.material_group, self.vertex, self.uv, self.normal, self.face)
    }
}

struct PartialMesh {
    partial_faces: Vec<PartialFace>,
    object_source: String,
    object_name: Option<String>,
    group_name: Option<String>,
    material_name: Option<String>,
}

pub fn load_obj(
    filepath: &str,
    material_arena: &mut Arena<Material>,
    texture_arena: &mut Arena<TextureMap>,
) -> LoadObjResult {
    let path = Path::new(&filepath);

    let parent_path = path.parent().unwrap();

    let lines = read_lines(path).unwrap();

    let object_source = Some(path.to_str().unwrap().to_string());

    let mut material_source: Option<String> = None;

    let mut object_name: Option<String> = None;
    let mut group_name: Option<String> = None;
    let mut material_name: Option<String> = None;

    let mut vertices: Vec<Vec3> = vec![];
    let mut normals: Vec<Vec3> = vec![];
    let mut uvs: Vec<Vec2> = vec![];

    let mut partial_faces: Vec<PartialFace> = vec![];
    let mut partial_meshes: Vec<PartialMesh> = vec![];

    let mut counts: LoadObjStats = Default::default();

    let end_signal: std::vec::IntoIter<Result<String, Error>> =
        vec![(Ok("usemtl __sentinel__".to_string()))].into_iter();

    let lines_safe = lines.chain(end_signal);

    for next_line in lines_safe {
        match next_line {
            Err(err) => {
                println!("Error reading next line: {}", err);

                continue;
            }
            Ok(line) => {
                let mut line_tokens = line.split_whitespace();

                let data_type = line_tokens.next();

                match data_type {
                    None => (),
                    Some(dt) => {
                        match dt {
                            // Comment
                            "#" => {
                                println!("# {}", line_tokens.next().unwrap_or_default());
                            }
                            // Geometric vertex, with (x, y, z, [w]) coordinates, w is optional and defaults to 1.0.
                            "v" => {
                                let vertex = parse_vertex(&mut line_tokens).unwrap();

                                vertices.push(vertex);

                                counts.vertex += 1;
                            }
                            // Texture (UV) coordinate, as (u, [v, w]), between 0 and 1. v, w are optional and default to 0.
                            "vt" => {
                                let uv = parse_vertex_uv(&mut line_tokens).unwrap();

                                uvs.push(uv);

                                counts.uv += 1;
                            }
                            // Vertex normal in (x,y,z) form; normal might not be a unit vector.
                            "vn" => {
                                let normal = parse_vertex_normal(&mut line_tokens).unwrap();

                                normals.push(normal);

                                counts.normal += 1;
                            }
                            // Parameter space vertex
                            "vp" => {
                                // TODO
                            }
                            // Polygonal face
                            "f" => {
                                let partial_face = parse_face(&mut line_tokens).unwrap();

                                partial_faces.push(partial_face);

                                counts.face += 1;
                            }
                            // Line element
                            "l" => {
                                // TODO
                            }
                            // External material reference
                            "mtllib" => {
                                let path = parse_mtllib(&mut line_tokens, parent_path).unwrap();

                                material_source = Some(path);

                                // println!("mtllib {}", material_source.as_ref().unwrap());
                            }
                            // Material group
                            "usemtl" => {
                                // If we were compiling a previous face group,
                                // package it into a Mesh before we continue.

                                let next_material_name =
                                    Some(line_tokens.next().unwrap().to_string());

                                if next_material_name != material_name {
                                    let mut partial_mesh = PartialMesh {
                                        partial_faces: vec![],
                                        object_source: object_source.as_ref().unwrap().clone(),
                                        object_name: object_name.clone(),
                                        group_name: group_name.clone(),
                                        material_name,
                                    };

                                    mem::swap(&mut partial_faces, &mut partial_mesh.partial_faces);

                                    partial_meshes.push(partial_mesh);
                                }

                                material_name = next_material_name;

                                // println!(
                                //     "\tusemtl {} for material group {}.",
                                //     material_name.as_ref().unwrap(),
                                //     group_name.as_ref().unwrap()
                                // );

                                counts.material_group += 1;
                            }
                            // Named object
                            "o" => {
                                object_name = Some(line_tokens.next().unwrap().to_string());

                                // println!("o {}", object_name.as_ref().unwrap());

                                counts.object += 1;
                            }
                            // Named object group
                            "g" => {
                                group_name = Some(line_tokens.next().unwrap().to_string());

                                // println!("g {}", group_name.as_ref().unwrap());

                                counts.group += 1;
                            }
                            // Smoothing group
                            "s" => {
                                // TODO
                            }
                            other => {
                                // Unrecognized prefix
                                println!("Unrecognized data type: {}", other)
                            }
                        }
                    }
                }
            }
        }
    }

    match &material_source {
        Some(src) => load_mtl(src, material_arena, texture_arena),
        None => (),
    }

    let geometry = Geometry {
        vertices: vertices.into_boxed_slice(),
        normals: normals.into_boxed_slice(),
        uvs: uvs.into_boxed_slice(),
    };

    let geometry_rc = Rc::new(geometry);

    let mut meshes: Vec<Mesh> = vec![];

    for partial_mesh in partial_meshes {
        let material = partial_mesh.material_name.as_ref().and_then(|name| {
            let material_slot_index = material_arena.entries.iter().position(|slot| match slot {
                Some(entry) => {
                    let material = &entry.item;

                    material.name == *name
                }
                None => false,
            });

            material_slot_index.map(|index| material_arena.get_handle(index).unwrap())
        });

        let mut mesh = Mesh::new(geometry_rc.clone(), partial_mesh.partial_faces, material);

        partial_mesh.object_name.clone_into(&mut mesh.object_name);

        mesh.object_source = Some(partial_mesh.object_source.to_owned());

        partial_mesh.group_name.clone_into(&mut mesh.group_name);

        meshes.push(mesh);
    }

    println!("{}", counts);
    println!("Parsed {} meshes.", meshes.len());

    LoadObjResult(geometry_rc, meshes)
}
