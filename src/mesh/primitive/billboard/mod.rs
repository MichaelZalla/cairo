use crate::{
    mesh::geometry::Geometry,
    texture::uv,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

use super::Face;

pub fn generate(position: Vec3, view_position: &Vec3, width: f32, height: f32) -> Geometry {
    // Computes basis vectors based on billboard position and view (camera) position.

    let world_up = vec3::UP;

    let forward = (position - *view_position).as_normal();

    let right = world_up.cross(forward).as_normal() * width / 2.0;

    let up = forward.cross(right).as_normal() * height / 2.0;

    let mut vertices: Vec<Vec3> = vec![
        // Top left
        (up - right),
        // Top right
        (up + right),
        // Bottom left
        (up * -1.0 - right),
        // Bottom right
        (up * -1.0 + right),
    ];

    // Bakes a world-space transform into the vertices.

    for i in 0..vertices.len() {
        vertices[i] += position;
    }

    let uvs: Vec<Vec2> = vec![
        uv::TOP_LEFT,
        uv::TOP_RIGHT,
        uv::BOTTOM_LEFT,
        uv::BOTTOM_RIGHT,
    ];

    let normals: Vec<Vec3> = vec![forward];

    let faces: Vec<Face> = vec![
        Face {
            // (top_right, bottom_left, top_left)
            vertices: (1, 2, 0),
            // (top_right, bottom_left, top_left)
            uvs: Some((1, 2, 0)),
            // (backward, backward, backward)
            normals: Some((0, 0, 0)),
        },
        Face {
            // (top_right, bottom_right, bottom_left)
            vertices: (1, 3, 2),
            // (top_right, bottom_right, bottom_left)
            uvs: Some((1, 3, 2)),
            // (backward, backward, backward)
            normals: Some((0, 0, 0)),
        },
    ];

    Geometry::new(vertices, uvs, normals, faces)
}
