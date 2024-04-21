use std::{path::Path, str::SplitWhitespace};

use crate::{
    mesh::Face,
    vec::{vec2::Vec2, vec3::Vec3},
};

pub fn parse_vertex<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<Vec3, String> {
    // `v  -0.512365 -40.559704 21.367237` (x y z)
    // `v  -0.512365 -40.559704 21.367237 50 255 0` (x y z r g b)

    let (x, y, z) = (
        tokens.next().unwrap().parse::<f32>().unwrap(),
        tokens.next().unwrap().parse::<f32>().unwrap(),
        tokens.next().unwrap().parse::<f32>().unwrap(),
    );

    Ok(Vec3 { x, y, z })
}

pub fn parse_vertex_uv<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<Vec2, String> {
    // `vt 0.500 1 [0]` (u v w?)

    let u = tokens.next().unwrap().parse::<f32>().unwrap();
    let mut v = 0.0;
    let mut w = 0.0;

    let result = tokens.next();

    match result {
        Some(value) => {
            v = value.parse::<f32>().unwrap();

            let result = tokens.next();

            match result {
                Some(value) => {
                    w = value.parse::<f32>().unwrap();
                }
                None => (),
            }
        }
        None => (),
    }

    Ok(Vec2 { x: u, y: v, z: w })
}

pub fn parse_vertex_normal<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<Vec3, String> {
    // `vn  0.000005 -34.698460 -17.753405` (x y z)

    let x = tokens.next().unwrap().parse::<f32>().unwrap();
    let y = tokens.next().unwrap().parse::<f32>().unwrap();
    let z = tokens.next().unwrap().parse::<f32>().unwrap();

    Ok(Vec3 { x, y, z })
}

pub fn parse_face<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<Face, String> {
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

    let mut v1_iter = tokens.next().unwrap().split("/");
    let mut v2_iter = tokens.next().unwrap().split("/");
    let mut v3_iter = tokens.next().unwrap().split("/");

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
                let v1_uv = v1_uv_index.unwrap().parse::<usize>().unwrap() - 1;
                let v2_uv = v2_uv_index.unwrap().parse::<usize>().unwrap() - 1;
                let v3_uv = v3_uv_index.unwrap().parse::<usize>().unwrap() - 1;

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

            let v1_n_raw = v1_normal_index.unwrap().parse::<usize>().unwrap();
            let v2_n_raw = v2_normal_index.unwrap().parse::<usize>().unwrap();
            let v3_n_raw = v3_normal_index.unwrap().parse::<usize>().unwrap();

            let v1_n = v1_n_raw - 1;
            let v2_n = v2_n_raw - 1;
            let v3_n = v3_n_raw - 1;

            face.normals = Some((v1_n, v2_n, v3_n));
        }
        None => (),
    }

    Ok(face)
}

pub fn parse_mtllib<'a>(
    tokens: &mut SplitWhitespace<'a>,
    parent_path: &Path,
) -> Result<String, String> {
    let mtl_filepath = tokens.next().unwrap();

    let mtl_path_relative = parent_path
        .join(mtl_filepath)
        .into_os_string()
        .into_string()
        .unwrap();

    let mtl_path_relative_str = mtl_path_relative.as_str();

    Ok(mtl_path_relative_str.to_string())
}

// pub fn parse_vertex<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<Vec3, String> {}

// pub fn parse_vertex<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<Vec3, String> {}
