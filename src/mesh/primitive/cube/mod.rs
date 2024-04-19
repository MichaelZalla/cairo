use crate::{
    mesh::geometry::Geometry,
    texture,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
    },
};

use super::Face;

pub fn generate(width: f32, height: f32, depth: f32) -> Geometry {
    // Generate vertices

    let front_top_left = Vec3 {
        x: -width / 2.0,
        y: height / 2.0,
        z: -depth / 2.0,
    };

    let front_top_right = Vec3 {
        x: width / 2.0,
        y: height / 2.0,
        z: -depth / 2.0,
    };

    let front_bottom_left = Vec3 {
        x: -width / 2.0,
        y: -height / 2.0,
        z: -depth / 2.0,
    };

    let front_bottom_right = Vec3 {
        x: width / 2.0,
        y: -height / 2.0,
        z: -depth / 2.0,
    };

    let mut back_top_left = front_top_left.clone();

    back_top_left.z += depth;

    let mut back_top_right = front_top_right.clone();

    back_top_right.z += depth;

    let mut back_bottom_left = front_bottom_left.clone();

    back_bottom_left.z += depth;

    let mut back_bottom_right = front_bottom_right.clone();

    back_bottom_right.z += depth;

    // Generate normals

    let forward = vec3::FORWARD;
    let backward = forward * -1.0;
    let up = vec3::UP;
    let down = up * -1.0;
    let left = vec3::LEFT;
    let right = left * -1.0;

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

    let uvs: Vec<Vec2> = vec![
        texture::uv::TOP_LEFT,     // 0
        texture::uv::TOP_RIGHT,    // 1
        texture::uv::BOTTOM_LEFT,  // 2
        texture::uv::BOTTOM_RIGHT, // 3
    ];

    let normals: Vec<Vec3> = vec![
        forward,  // 0
        backward, // 1
        up,       // 2
        down,     // 3
        left,     // 4
        right,    // 5
    ];

    // Generate faces

    let mut faces: Vec<Face> = vec![];

    // Front face

    let front_face_1 = Face {
        // (front_top_right, front_bottom_left, front_top_left)
        vertices: (1, 2, 0),
        // (top_right, bottom_left, top_left)
        uvs: Some((1, 2, 0)),
        // (backward, backward, backward)
        normals: Some((1, 1, 1)),
    };

    let front_face_2 = Face {
        // (front_top_right, front_bottom_right, front_bottom_left)
        vertices: (1, 3, 2),
        // (top_right, bottom_right, bottom_left)
        uvs: Some((1, 3, 2)),
        // (backward, backward, backward)
        normals: Some((1, 1, 1)),
    };

    faces.push(front_face_1);
    faces.push(front_face_2);

    // Back face

    let back_face_1 = Face {
        // (back_bottom_left, back_top_right, back_top_left)
        vertices: (6, 5, 4),
        // (bottom_right, top_left, top_right)
        uvs: Some((3, 0, 1)),
        // (forward, forward, forward)
        normals: Some((0, 0, 0)),
    };

    let back_face_2 = Face {
        // (back_bottom_left, back_bottom_right, back_top_right)
        vertices: (6, 7, 5),
        // (bottom_right, bottom_left, top_left)
        uvs: Some((3, 2, 0)),
        // (forward, forward, forward)
        normals: Some((0, 0, 0)),
    };

    faces.push(back_face_1);
    faces.push(back_face_2);

    // Top face

    let top_face_1 = Face {
        // (back_top_right, front_top_left, back_top_left)
        vertices: (5, 0, 4),
        // (top_right, bottom_left, top_left)
        uvs: Some((1, 2, 0)),
        // (up, up, up)
        normals: Some((2, 2, 2)),
    };

    let top_face_2 = Face {
        // (back_top_right, front_top_right, front_top_left)
        vertices: (5, 1, 0),
        // (top_right, bottom_right, bottom_left)
        uvs: Some((1, 3, 2)),
        // (up, up, up)
        normals: Some((2, 2, 2)),
    };

    faces.push(top_face_1);
    faces.push(top_face_2);

    // Bottom face

    let bottom_face_1 = Face {
        // (front_bottom_left, back_bottom_right, back_bottom_left)
        vertices: (2, 7, 6),
        // (top_left, bottom_right, bottom_left)
        uvs: Some((0, 3, 2)),
        // (down, down, down)
        normals: Some((3, 3, 3)),
    };

    let bottom_face_2 = Face {
        // (front_bottom_left, front_bottom_right, back_bottom_right)
        vertices: (2, 3, 7),
        // (top_left, top_right, bottom_right)
        uvs: Some((0, 1, 3)),
        // (down, down, down)
        normals: Some((3, 3, 3)),
    };

    faces.push(bottom_face_1);
    faces.push(bottom_face_2);

    // Left face

    let left_face_1 = Face {
        // (front_top_left, back_bottom_left, back_top_left)
        vertices: (0, 6, 4),
        // (top_right, bottom_left, top_left)
        uvs: Some((1, 2, 0)),
        // (left, left, left)
        normals: Some((4, 4, 4)),
    };

    let left_face_2 = Face {
        // (front_top_left, front_bottom_left, back_bottom_left)
        vertices: (0, 2, 6),
        // (top_right, bottom_right, bottom_left)
        uvs: Some((1, 3, 2)),
        // (left, left, left)
        normals: Some((4, 4, 4)),
    };

    faces.push(left_face_1);
    faces.push(left_face_2);

    // Right face

    let right_face_1 = Face {
        // (back_bottom_right, front_bottom_right, front_top_right)
        vertices: (7, 3, 1),
        // (bottom_right, bottom_left, top_left)
        uvs: Some((3, 2, 0)),
        // (right, right, right)
        normals: Some((5, 5, 5)),
    };

    let right_face_2 = Face {
        // (front_top_right back_top_right, back_bottom_right)
        vertices: (1, 5, 7),
        // (top_left, top_right, bottom_right)
        uvs: Some((0, 1, 3)),
        // (right, right, right)
        normals: Some((5, 5, 5)),
    };

    faces.push(right_face_1);
    faces.push(right_face_2);

    let mut geometry = Geometry::new(vertices, uvs, normals, faces);

    geometry.object_name = Some("cube".to_string());

    geometry
}
