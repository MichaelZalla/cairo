use crate::vec::{vec2::Vec2, vec3::Vec3};

use super::Mesh;

pub fn make_box(width: f32, height: f32, depth: f32) -> Mesh {
    let front_top_left = Vec3 {
        x: -width / 2.0,
        y: height / 2.0,
        z: depth / 2.0,
    };

    let front_top_right = Vec3 {
        x: width / 2.0,
        y: height / 2.0,
        z: depth / 2.0,
    };

    let front_bottom_left = Vec3 {
        x: -width / 2.0,
        y: -height / 2.0,
        z: depth / 2.0,
    };

    let front_bottom_right = Vec3 {
        x: width / 2.0,
        y: -height / 2.0,
        z: depth / 2.0,
    };

    let mut back_top_left = front_top_left.clone();

    back_top_left.z -= depth;

    let mut back_top_right = front_top_right.clone();

    back_top_right.z -= depth;

    let mut back_bottom_left = front_bottom_left.clone();

    back_bottom_left.z -= depth;

    let mut back_bottom_right = front_bottom_right.clone();

    back_bottom_right.z -= depth;

    let vertices: Vec<Vec3> = vec![
        front_top_left,     // 0
        front_top_right,    // 1
        front_bottom_left,  // 2
        front_bottom_right, // 3
        back_top_left,      // 4
        back_top_right,     // 5
        back_bottom_left,   // 6
        back_bottom_right,  // 7
    ];

    let mut face_vertex_indices: Vec<(usize, usize, usize)> = vec![];

    // Front face

    face_vertex_indices.push((0, 2, 1));
    face_vertex_indices.push((2, 3, 1));

    // Back face

    face_vertex_indices.push((4, 5, 6));
    face_vertex_indices.push((5, 7, 6));

    // Top face

    face_vertex_indices.push((4, 0, 5));
    face_vertex_indices.push((0, 1, 5));

    // Bottom face

    face_vertex_indices.push((6, 7, 2));
    face_vertex_indices.push((7, 3, 2));

    // Left face

    face_vertex_indices.push((4, 6, 0));
    face_vertex_indices.push((6, 2, 0));

    // Right face

    face_vertex_indices.push((1, 3, 7));
    face_vertex_indices.push((7, 5, 1));

    // Generates dummy texture coordinates
    // @TODO Generate correct vertex texture coordinates!

    let vertex_uv_coordinates: Vec<Vec2> = vec![];

    // Generates dummy normals
    // @TODO Generate correct vertex normals!

    let mut vertex_normals: Vec<Vec3> = vec![];

    for _ in vertices.as_slice() {
        vertex_normals.push(Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        })
    }

    let face_vertex_uv_coordinate_indices: Vec<(usize, usize, usize)> = vec![];

    let face_vertex_normal_indices: Vec<(usize, usize, usize)> = vec![];

    return Mesh::new(
        vertices,
        vertex_uv_coordinates,
        vertex_normals,
        face_vertex_indices,
        face_vertex_uv_coordinate_indices,
        face_vertex_normal_indices,
    );
}
