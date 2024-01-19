use crate::{
    scene::camera::Camera,
    texture::uv,
    vec::{vec2::Vec2, vec3::Vec3},
};

use super::{Face, Mesh};

pub fn generate(camera: &Camera, width: f32, height: f32) -> Mesh {
    let forward = camera.get_forward();
    let up = (camera.get_up().as_normal()) * height / 2.0;
    let right = (camera.get_right().as_normal()) * width / 2.0;

    let vertices: Vec<Vec3> = vec![
        // Top left
        (up - right),
        // Top right
        (up + right),
        // Bottom left
        (up * -1.0 - right),
        // Bottom right
        (up * -1.0 + right),
    ];

    let uvs: Vec<Vec2> = vec![
        uv::TOP_LEFT,
        uv::TOP_RIGHT,
        uv::BOTTOM_LEFT,
        uv::BOTTOM_RIGHT,
    ];

    let normals: Vec<Vec3> = vec![forward * -1.0];

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

    let mut mesh = Mesh::new(vertices, uvs, normals, faces);

    mesh.object_name = "billboard".to_string();

    return mesh;
}
