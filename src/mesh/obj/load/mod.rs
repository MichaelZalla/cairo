use std::{fmt, io::Error, mem, path::Path, rc::Rc};

use crate::{
    fs::read_lines,
    material::{cache::MaterialCache, mtl::load_mtl},
    mesh::{
        geometry::Geometry,
        obj::parse::{
            parse_face, parse_mtllib, parse_vertex, parse_vertex_normal, parse_vertex_uv,
        },
        Face, Mesh,
    },
    resource::arena::Arena,
    texture::map::TextureMap,
};

pub struct LoadObjResult(pub Rc<Geometry>, pub Vec<Mesh>, pub Option<MaterialCache>);

#[derive(Default, Debug)]
struct ObjDataTypeCounts {
    object: usize,
    group: usize,
    material_group: usize,
    vertex: usize,
    uv: usize,
    normal: usize,
    face: usize,
}

impl fmt::Display for ObjDataTypeCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counted {} objects, {} groups, {} material groups, {} vertices, {} UVs, {} normals, and {} faces.", self.object, self.group, self.material_group, self.vertex, self.uv, self.normal, self.face)
    }
}

pub fn load_obj(filepath: &str, texture_arena: &mut Arena<TextureMap>) -> LoadObjResult {
    let path = Path::new(&filepath);

    let parent_path = path.parent().unwrap();

    let lines = read_lines(path).unwrap();

    let object_source = Some(path.to_str().unwrap().to_string());

    let mut material_source: Option<String> = None;

    let mut object_name: Option<String> = None;
    let mut group_name: Option<String> = None;
    let mut material_name: Option<String> = None;

    let mut geometry = Geometry {
        vertices: vec![],
        normals: vec![],
        uvs: vec![],
    };

    let mut faces: Vec<Face> = vec![];

    let mut meshes: Vec<Mesh> = vec![];

    let mut counts: ObjDataTypeCounts = Default::default();

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

                                geometry.vertices.push(vertex);

                                counts.vertex += 1;
                            }
                            // Texture (UV) coordinate, as (u, [v, w]), between 0 and 1. v, w are optional and default to 0.
                            "vt" => {
                                let uv = parse_vertex_uv(&mut line_tokens).unwrap();

                                geometry.uvs.push(uv);

                                counts.uv += 1;
                            }
                            // Vertex normal in (x,y,z) form; normal might not be a unit vector.
                            "vn" => {
                                let normal = parse_vertex_normal(&mut line_tokens).unwrap();

                                geometry.normals.push(normal);

                                counts.normal += 1;
                            }
                            // Parameter space vertex
                            "vp" => {
                                // TODO
                            }
                            // Polygonal face
                            "f" => {
                                let face = parse_face(&mut line_tokens).unwrap();

                                faces.push(face);

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
                                    let mut faces_for_material: Vec<Face> = vec![];

                                    mem::swap(&mut faces, &mut faces_for_material);

                                    let mut mesh =
                                        Mesh::new(None, faces_for_material, material_name);

                                    mesh.object_name = object_name.clone();
                                    mesh.object_source = object_source.clone();
                                    mesh.group_name = group_name.clone();

                                    meshes.push(mesh);
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

    println!("{}", counts);
    println!("Parsed {} meshes.", meshes.len());

    let geometry_rc = Rc::new(geometry);

    for mesh in meshes.iter_mut() {
        // Sets a reference to the shared geometry from this OBJ file.

        mesh.geometry = Some(geometry_rc.clone());
    }

    let mut materials: Option<MaterialCache> = None;

    match &material_source {
        Some(src) => materials = Some(load_mtl(src, texture_arena)),
        None => (),
    }

    LoadObjResult(geometry_rc, meshes, materials)
}
