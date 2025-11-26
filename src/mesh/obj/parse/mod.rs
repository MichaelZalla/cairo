use std::{path::Path, str::SplitWhitespace};

use crate::{
    mesh::PartialFace,
    vec::{vec2::Vec2, vec3::Vec3},
};

pub fn parse_vertex(tokens: &mut SplitWhitespace<'_>) -> Result<Vec3, String> {
    // `v  -0.512365 -40.559704 21.367237` (x y z)
    // `v  -0.512365 -40.559704 21.367237 50 255 0` (x y z r g b)

    let (x, y, z) = (
        tokens.next().unwrap().parse::<f32>().unwrap(),
        tokens.next().unwrap().parse::<f32>().unwrap(),
        tokens.next().unwrap().parse::<f32>().unwrap(),
    );

    Ok(Vec3 { x, y, z })
}

pub fn parse_vertex_uv(tokens: &mut SplitWhitespace<'_>) -> Result<Vec2, String> {
    // `vt 0.500 1 [0]` (u v w?)

    let u = tokens.next().unwrap().parse::<f32>().unwrap();
    let mut v = 0.0;
    let mut w = 0.0;

    let result = tokens.next();

    if let Some(value) = result {
        v = value.parse::<f32>().unwrap();

        let result = tokens.next();

        if let Some(value) = result {
            w = value.parse::<f32>().unwrap();
        }
    }

    Ok(Vec2 { x: u, y: v, z: w })
}

pub fn parse_vertex_normal(tokens: &mut SplitWhitespace<'_>) -> Result<Vec3, String> {
    // `vn  0.000005 -34.698460 -17.753405` (x y z)

    let x = tokens.next().unwrap().parse::<f32>().unwrap();
    let y = tokens.next().unwrap().parse::<f32>().unwrap();
    let z = tokens.next().unwrap().parse::<f32>().unwrap();

    Ok(Vec3 { x, y, z })
}

pub fn parse_face(tokens: &mut SplitWhitespace<'_>) -> Result<PartialFace, String> {
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

    let mut partial_face: PartialFace = Default::default();

    let mut v1_iter = tokens.next().unwrap().split('/');
    let mut v2_iter = tokens.next().unwrap().split('/');
    let mut v3_iter = tokens.next().unwrap().split('/');

    partial_face.vertices = [
        v1_iter.next().unwrap().parse::<usize>().unwrap() - 1,
        v2_iter.next().unwrap().parse::<usize>().unwrap() - 1,
        v3_iter.next().unwrap().parse::<usize>().unwrap() - 1,
    ];

    let v1_uv_index = v1_iter.next();
    let v2_uv_index = v2_iter.next();
    let v3_uv_index = v3_iter.next();

    if let Some(index) = v1_uv_index
        && !index.is_empty()
    {
        let v1_uv = v1_uv_index.unwrap().parse::<usize>().unwrap() - 1;
        let v2_uv = v2_uv_index.unwrap().parse::<usize>().unwrap() - 1;
        let v3_uv = v3_uv_index.unwrap().parse::<usize>().unwrap() - 1;

        partial_face.uvs = Some([v1_uv, v2_uv, v3_uv]);
    }

    let v1_normal_index = v1_iter.next();

    if let Some(v1_normal_index) = v1_normal_index {
        let v2_normal_index = v2_iter.next();
        let v3_normal_index = v3_iter.next();

        let v1_n_raw = v1_normal_index.parse::<usize>().unwrap();
        let v2_n_raw = v2_normal_index.unwrap().parse::<usize>().unwrap();
        let v3_n_raw = v3_normal_index.unwrap().parse::<usize>().unwrap();

        let v1_n = v1_n_raw - 1;
        let v2_n = v2_n_raw - 1;
        let v3_n = v3_n_raw - 1;

        partial_face.normals = Some([v1_n, v2_n, v3_n]);
    }

    Ok(partial_face)
}

pub fn parse_mtllib(
    tokens: &mut SplitWhitespace<'_>,
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
